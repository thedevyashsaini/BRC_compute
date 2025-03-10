// Function to delete the folder if it exists
import * as fs from "node:fs";
import * as path from "path";
import dotenv from "dotenv";
import type { Octokit } from "octokit";
import type { PostgresJsDatabase } from "drizzle-orm/postgres-js";
import { submissionTable } from "../db/schema.js";
import type { Submission } from "../db/schema.js";
import { eq } from "drizzle-orm";

dotenv.config();

export function deleteFolderIfExists(folderPath: string) {
  if (fs.existsSync(folderPath)) {
    fs.rmSync(folderPath, { recursive: true, force: true });
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
        .set({ commit_status: state, commit_description: description })
        .where(eq(submissionTable.id, this.submission.id));

      try {
        await this.octokit.rest.repos.createCommitStatus({
          owner: this.repository.owner.login,
          repo: this.repository.name,
          sha: this.after,
          state: state,
          description: description.length > 140 ? description.substring(0, 140) : description,
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

export async function listDirRecursive(dir: string, indent = 0): Promise<string> {
  let result = '';
  const files = await fs.promises.readdir(dir);
  
  for (const file of files) {
    if (file === '.git') continue;
    
    const filePath = path.join(dir, file);
    const stats = await fs.promises.stat(filePath);
    const indentation = ' '.repeat(indent * 2);
    
    if (stats.isDirectory()) {
      result += `\t${indentation}📁 ${file}/\n`;
      result += await listDirRecursive(filePath, indent + 1);
    } else {
      result += `\t${indentation}📄 ${file}\n`;
    }
  }
  
  return result;
}