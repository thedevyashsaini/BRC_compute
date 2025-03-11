import { getDB } from "../db/index.js";
import { submissionTable, userTable } from "../db/schema.js";
import { eq } from "drizzle-orm";
import { GitHubService } from "../services/github-service.js";
import { DockerService } from "../services/docker-service.js";
import { BenchmarkService } from "../services/benchmark-service.js";
import { Submission } from "../models/submission.js";
import { deleteFolderIfExists } from "../utils/helper.js";
import { TEST_LEVEL } from "../config/app-config.js";

export class SubmissionProcessor {
  private db: any;
  private githubService: GitHubService;
  private dockerService: DockerService;
  private benchmarkService: BenchmarkService;

  constructor() {
    this.githubService = new GitHubService();
    this.dockerService = new DockerService();
    this.benchmarkService = new BenchmarkService();
  }

  async process(data: any): Promise<void> {
    this.db = await getDB();

    const { repository, installation, after } = data;

    console.log(" [x] Processing submission for repository:", repository.name);

    // Find the user for this repo
    const user = await this.findUser(repository.id);

    // Create a submission record
    const submission = await this.createSubmission(user.id, after);

    // Initialize the submission handler
    const submissionHandler = new Submission(
      repository,
      installation.id,
      after,
      submission,
      this.db
    );

    await submissionHandler.initialize();

    try {
      // Clone repositories
      await submissionHandler.cloneRepositories();

      // Build Docker image
      await this.dockerService.buildImage(
        submissionHandler.containerName,
        submissionHandler.getFolderPath()
      );

      await submissionHandler.updateStatus(
        "pending",
        "Segssy, build succeeded, running benchmarks..."
      );

      // Run benchmarks
      await this.dockerService.runBenchmarks(
        submissionHandler.containerName,
        submissionHandler.getFolderPath(),
        TEST_LEVEL || "1"
      );

      // Get benchmark results
      const outputPath = await this.dockerService.copyOutputFromContainer(
        submissionHandler.containerName,
        submissionHandler.getFolderPath()
      );

      const benchResults = await this.benchmarkService.extractResults(
        outputPath
      );

      // Update submission with results
      await this.db
        .update(submissionTable)
        .set({
          commit_status: "success",
          commit_description: `IDK how, but the whole thing workd, ${this.benchmarkService.formatRuntimeDescription(
            benchResults.parsed.mean
          )}`,
          runtime: benchResults.parsed.mean.toString(),
          parsed_json: benchResults.parsed,
          raw_json: benchResults.raw,
        })
        .where(eq(submissionTable.id, submission.id));

      // Update commit status
      await this.githubService.updateCommitStatus(
        await this.githubService.getOctokit(installation.id),
        repository,
        after,
        "success",
        `IDK how, but the whole thing workd, ${this.benchmarkService.formatRuntimeDescription(
          benchResults.parsed.mean
        )}`
      );

      console.log(" [x] Process completed successfully! Slave can Sleep!");
    } catch (error) {
      console.error(" [-] Error during processing:", error);
      await submissionHandler.updateStatus("error", `Failed: ${error}`);
      throw error;
    } finally {
      // Clean up
      try {
        deleteFolderIfExists(submissionHandler.getFolderPath());
        await this.dockerService.cleanup(submissionHandler.containerName);
      } catch (cleanupError) {
        console.error(" [-] Cleanup error:", cleanupError);
      }
    }
  }

  private async findUser(repoId: string) {
    const user = await this.db
      .select({
        id: userTable.id,
        email: userTable.email,
      })
      .from(userTable)
      .where(eq(userTable.github_repo, repoId));

    if (!user || user.length === 0) {
      throw new Error("User not found for repository");
    }

    return user[0];
  }

  private async createSubmission(userId: string, commitHash: string) {
    const submissions = await this.db
      .insert(submissionTable)
      .values({
        user_id: userId,
        commit_hash: commitHash,
        commit_status: "Initializing",
        commit_description: "Request pulled by worker",
      })
      .returning();

    if (!submissions || submissions.length === 0) {
      throw new Error("Failed to create submission record");
    }

    return submissions[0];
  }
}
