import path from "path";
import { promisify } from "util";
import { exec as execCallback } from "child_process";
import type { Octokit } from "octokit";
import type { PostgresJsDatabase } from "drizzle-orm/postgres-js";
import { submissionTable } from "../db/schema.js";
import type { Submission as SubmissionTable } from "../db/schema.js";
import { eq } from "drizzle-orm";
import { GitHubService } from "../services/github-service.js";
import { deleteFolderIfExists } from "../utils/helper.js";
import { BASE_DIR, WORKER_NAME } from "../config/app-config.js";

const exec = promisify(execCallback);

type CommitState = "pending" | "success" | "failure" | "error";

export class Submission {
  private folderPath: string;
  private octokit: any;
  private commitUpdater!: CommitUpdater;
  private githubService: GitHubService;

  constructor(
    private repository: any,
    private installationId: number,
    private commitSha: string,
    private dbSubmission: any,
    private db: any
  ) {
    this.githubService = new GitHubService();
    this.folderPath = path.join(BASE_DIR, `src/${this.containerName}`);
  }

  get containerName(): string {
    return `${this.repository.owner.name}_${this.repository.name}`.toLowerCase();
  }

  async initialize(): Promise<void> {
    this.octokit = await this.githubService.getOctokit(this.installationId);
    this.commitUpdater = new CommitUpdater(
      this.octokit,
      this.db,
      this.repository,
      this.commitSha,
      this.dbSubmission
    );

    await this.commitUpdater.run(
      "pending",
      `Holon buddy, ${WORKER_NAME} here! Lemme cook...`
    );

    deleteFolderIfExists(this.folderPath);
  }

  async cloneRepositories(): Promise<void> {
    const userToken = await this.githubService.getInstallationToken(
      this.installationId,
      this.repository.name
    );

    const brcToken = await this.githubService.getBrcToken();

    const cloneUrl = this.repository.clone_url.replace(
      "https://",
      `https://x-access-token:${userToken}@`
    );

    const tempPath = path.join(BASE_DIR, `src/temp_${this.containerName}`);
    deleteFolderIfExists(tempPath);

    await exec(`git clone ${cloneUrl} ${tempPath}`);

    const brcCloneUrl = `https://x-access-token:${brcToken}@github.com/thedevyashsaini/BRC.git`;
    await exec(`git clone ${brcCloneUrl} ${this.folderPath}`);

    await exec(`cp -rf ${tempPath}/src/* ${this.folderPath}/src/`);
    deleteFolderIfExists(tempPath);

    await this.commitUpdater.run("pending", `Got your code, building...`);
  }

  async updateStatus(state: CommitState, description: string): Promise<void> {
    await this.commitUpdater.run(state, description);
  }

  getFolderPath(): string {
    return this.folderPath;
  }
}

export class CommitUpdater {
  private octokit: Octokit;
  private db: PostgresJsDatabase<any>;
  private repository: Record<string, any>;
  private after: string;
  private submission: SubmissionTable | undefined;

  constructor(
    octokit: Octokit,
    db: PostgresJsDatabase<any>,
    repository: Record<string, any>,
    after: string,
    submission: SubmissionTable | undefined
  ) {
    this.octokit = octokit;
    this.db = db;
    this.repository = repository;
    this.after = after;
    this.submission = submission;
  }

  async run(state: CommitState, description: string): Promise<void> {
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
          description:
            description.length > 140
              ? description.substring(0, 140)
              : description,
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
