// Function to delete the folder if it exists
import * as fs from "node:fs";
import dotenv from "dotenv";
import type { Octokit } from "octokit";
import type { PostgresJsDatabase } from "drizzle-orm/postgres-js";
import { submissionTable } from "../db/schema.js";
import type { Submission } from "../db/schema.js";
import { eq } from "drizzle-orm";

dotenv.config();

export function deleteFolderIfExists(folderPath: string) {
  if (fs.existsSync(folderPath)) {
    fs.rmdirSync(folderPath, { recursive: true });
    console.log(` [x] Folder '${folderPath}' deleted successfully.`);
  } else {
    console.log(` [x] Folder '${folderPath}' does not exist.`);
  }
}

export class CommitUpdater {
  private octokit: Octokit;
  private db: PostgresJsDatabase<any>;
  private repository: Record<string, any>;
  private after: string;
  private submission: Submission | undefined;

  constructor(
    octokit: Octokit,
    db: PostgresJsDatabase<any>,
    repository: Record<string, any>,
    after: string,
    submission: Submission | undefined
  ) {
    this.octokit = octokit;
    this.db = db;
    this.repository = repository;
    this.after = after;
    this.submission = submission;
  }

  async run(state: "pending" | "error" | "failure" | "success", description: string): Promise<void> {
    await this.db.transaction(async (tx) => {
      if (!this.submission) {
        console.error(" [-] Submission not found");
        throw new Error("Submission not found");
      }

      await tx
        .update(submissionTable)
        .set({ commit_status: state + " | " + description })
        .where(eq(submissionTable.id, this.submission.id));

      try {
        await this.octokit.rest.repos.createCommitStatus({
          owner: this.repository.owner.login,
          repo: this.repository.name,
          sha: this.after,
          state: "pending",
          description: `Holon buddy, ${process.env.worker_name} here! Lemme cook...`,
        });
      } catch (error) {
        console.error(
          " [-] GitHub API call failed, rolling back transaction:",
          error
        );
        throw new Error("GitHub API call failed");
      }
    });
  }
}