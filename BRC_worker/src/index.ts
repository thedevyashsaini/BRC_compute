import amqp from "amqplib/callback_api.js";
import { App } from "octokit";
import { dirname } from "path";
import { fileURLToPath } from "node:url";
import * as path from "path";
import * as fs from "fs";
import { CommitUpdater, deleteFolderIfExists } from "./functions/helper.js";
import { getDB } from "./db/index.js";
import { submissionTable, userTable } from "./db/schema.js";
import { eq } from "drizzle-orm";
import { promisify } from "util";
import { exec as execCallback } from "child_process";
import { type BenchmarkStats } from "./types/benchmark.js";

const exec = promisify(execCallback);

process.chdir(dirname(fileURLToPath(import.meta.url)));
const currentDirectory = dirname(fileURLToPath(import.meta.url));

const privateKey = Buffer.from(
  process.env.GITHUB_PRIVATE_KEY!,
  "base64"
).toString("utf8");

const githubApp = new App({
  appId: process.env.GITHUB_APP_ID!,
  privateKey: privateKey,
});

amqp.connect("amqp://rabbitmq", function (error0, connection) {
  if (error0) {
    throw error0;
  }
  connection.createChannel(function (error1, channel) {
    if (error1) {
      throw error1;
    }
    var queue = "proposal";

    channel.assertQueue(queue, {
      durable: false,
    });

    console.log(" [*] Waiting for messages in %s. To exit press CTRL+C", queue);

    channel.consume(
      queue,
      async function (msg) {
        const db = await getDB();

        try {
          const { from, data } = JSON.parse(msg?.content.toString() || "{}");
          console.log(" [x] Received task from %s", from);
          console.log(" [x] Received task: %s", data.toString());

          if (!from || !data) {
            console.error(" [-] Invalid task");
            channel.nack(msg!, false, false);
            return;
          }

          const { repository, installation, after } = data;

          const octokit = await githubApp.getInstallationOctokit(
            installation.id
          );

          console.log(" [x] Got octokit");

          const user = await db
            .select({
              id: userTable.id,
              email: userTable.email,
            })
            .from(userTable)
            .where(eq(userTable.github_repo, repository.id));

          if (!user || user.length === 0) {
            console.error(" [-] Fking user not found");
            channel.nack(msg!, false, false);
            return;
          }

          console.log(" [x] Got repo owner from db");

          const submissions = await db
            .insert(submissionTable)
            .values({
              user_id: user[0]?.id || "",
              commit_hash: after,
              commit_status: "Initializing",
            })
            .returning();

          if (!submissions || submissions.length === 0) {
            console.error(" [-] Fking submission not found");
            channel.nack(msg!, false, false);
            return;
          }

          const submission = submissions[0];

          console.log(" [x] Inserted submission to db");

          const commitUpdater = new CommitUpdater(
            octokit,
            db,
            repository,
            after,
            submission
          );

          commitUpdater
            .run(
              "pending",
              `Holon buddy, ${process.env.WORKER_NAME} here! Lemme cook...`
            )
            .then(() => {
              console.log(" [x] Updated initial commit status for that moron");
            });

          const containerName =
            `${repository.owner.name}_${repository.name}`.toLowerCase();

          const folderPath = path.join(
            dirname(currentDirectory),
            `src/${containerName}`
          );
          deleteFolderIfExists(folderPath);

          console.log(" [*] Initiating clone and build");

          const {
            data: { token: installationToken },
          } = await octokit.request(
            `POST /app/installations/${installation.id}/access_tokens`,
            {
              installation_id: installation.id,
              repositories: [repository.name],
              permissions: {
                contents: "read",
              },
              headers: {
                "X-GitHub-Api-Version": "2022-11-28",
              },
            }
          );

          console.log(
            ` [x] Got installation token (IDK for what): ${installationToken}`
          );

          const cloneUrl = repository.clone_url.replace(
            "https://",
            `https://x-access-token:${installationToken}@`
          ) as string;

          try {
            console.log(` [x] Git clone ${cloneUrl}`);
            const tempPath = path.join(dirname(currentDirectory), `src/temp_${containerName}`);
            deleteFolderIfExists(tempPath);

            console.log(` [x] Git clone user repo to temp dir: ${cloneUrl}`);
            const { stdout: tempCloneOutput } = await exec(
              `git clone ${cloneUrl} ${tempPath}`
            );
            console.log(` [x] Git clone output -> ${tempCloneOutput}`);

            const brcCloneUrl = `https://x-access-token:${installationToken}@github.com/SteakFisher/BRC.git`;

            console.log(` [x] Git clone BRC repo to final dir: ${brcCloneUrl}`);
            const { stdout: brcCloneOutput } = await exec(
              `git clone ${brcCloneUrl} ${folderPath}`
            );
            console.log(` [x] BRC clone output -> ${brcCloneOutput}`);

            console.log(` [x] Copying user src files over BRC src files`);
            await exec(
              `cp -rf ${tempPath}/src/* ${folderPath}/src/ 2>/dev/null || true`
            );

            console.log(` [x] Cleaning up temp directory`);
            deleteFolderIfExists(tempPath); 

            await commitUpdater.run("pending", `Got your code, building...`);

            const { stdout: buildOutput } = await exec(
              `cd ${folderPath} && docker build -q -t ${containerName} .`
            );
            console.log(` [x] Docker build -> ${buildOutput}`);

            await commitUpdater.run(
              "pending",
              `Segssy, build succeeded, running benchmarks...`
            );
          } catch (error: any) {
            await commitUpdater.run(
              "error",
              "Clone or build fked up: " + error.toString()
            );
            throw new Error(`Clone or build fked up: ${error}`);
          }

          let benchData: {
            parsed: BenchmarkStats;
            raw: Record<any, any>;
          };

          try {
            console.log(" [x] Build succeeded...");

            console.log(" [x] Running benchmarks...");
            await exec(
              `LEVEL=100 CONTAINER_NAME=${containerName} docker compose up`
            );

            console.log(" [x] Fetching benchmark results...");

            try {
              await fs.promises.unlink("./status.json");
              await fs.promises.unlink("./bench.json");
              await fs.promises.unlink("./bench_parsed.json");
            } catch (error: any) {
              console.log(" [-] Failed to delete benchmark files.:", error.message);
            }

            let copySuccess = {
              status: false,
              bench: false,
              bench_parsed: false,
            };

            try {
              await exec(
                `docker cp temp_${containerName}:/usr/src/app/output/status.json ./status.json`
              );
              copySuccess.status = true;
              console.log(" [x] Successfully copied status.json");
            } catch (error: any) {
              console.error(" [-] Failed to copy status.json:", error.message);
            }

            if (!copySuccess.status) {
              await commitUpdater.run(
                "error",
                "Benchmark failed: status.json not found"
              );
              throw new Error("Benchmark failed: status.json not found");
            }

            const status: { success: boolean; message: string } = JSON.parse(
              await fs.promises.readFile("./status.json", "utf-8")
            );
            console.log(" [x] Status file content:", status);

            if (!status.success) {
              await commitUpdater.run(
                "failure",
                "Benchmark failed: " +
                  status.message +
                  ". If you feel like it's an internal error, please contact the maintainers."
              );
              throw new Error(`Benchmark failed: ${status.message}`);
            }

            try {
              await exec(
                `docker cp temp_${containerName}:/usr/src/app/output/bench.json ./bench.json`
              );
              copySuccess.bench = true;
              console.log(" [x] Successfully copied bench.json");
            } catch (error: any) {
              console.log(" [-] Failed to copy bench.json:", error.message);
            }

            try {
              await exec(
                `docker cp temp_${containerName}:/usr/src/app/output/bench_parsed.json ./bench_parsed.json`
              );
              copySuccess.bench_parsed = true;
              console.log(" [x] Successfully copied bench_parsed.json");
            } catch (error: any) {
              console.log(
                " [-] Failed to copy bench_parsed.json:",
                error.message
              );
            }

            if (!copySuccess.bench || !copySuccess.bench_parsed) {
              await commitUpdater.run(
                "error",
                "Benchmark failed: bench.json or bench_parsed.json not found"
              );
              throw new Error(
                "Benchmark failed: bench.json or bench_parsed.json not found"
              );
            }

            console.log(" [x] Benchmark files copied successfully");

            benchData = {
              parsed: JSON.parse(
                await fs.promises.readFile("./bench_parsed.json", "utf-8")
              ),
              raw: JSON.parse(
                await fs.promises.readFile("./bench_parsed.json", "utf-8")
              ),
            };

            console.log(" [x] Parsed benchmark Data: ", benchData.parsed);

            console.log(" [x] Stopping & removing container...");
            await exec(`docker rm -f temp_${containerName}`);
          } catch (error: any) {
            await commitUpdater.run(
              "error",
              "Test and benchmark failed: " + error.toString()
            );
            throw new Error(`Test and benchmark fked up: ${error}`);
          }

          if (!benchData || !benchData.parsed || !benchData.raw) { 
            await commitUpdater.run(
              "error",
              "That mf benchmark data ran away somewhere"
            );
            throw new Error("That mf benchmark data ran away somewhere");
          }

          const { parsed: parsed_bench, raw: raw_bench } = benchData;

          await db
            .update(submissionTable)
            .set({
              commit_status: `success | IDK how, but the whole thing workd, runtime: ${parsed_bench.mean} ms.`,
              runtime: parsed_bench.mean.toString(),
              parsed_json: parsed_bench,
              raw_json: raw_bench,
            })
            .where(eq(submissionTable.id, submission?.id || ""));

          await octokit.rest.repos.createCommitStatus({
            owner: repository.owner.login,
            repo: repository.name,
            sha: after,
            state: "success",
            description: `IDK how, but the whole thing workd, runtime: ${parsed_bench.mean} ms.`,
          });
          channel.ack(msg!);
          return;
        } catch (error) {
          console.error(" [-] Some shit went wrong: %s", error);
          channel.nack(msg!, false, false);
          return;
        }
      },
      {
        noAck: false,
      }
    );
  });
});