import amqp from "amqplib/callback_api.js";
import { App } from "octokit";
import { dirname } from "path";
import { fileURLToPath } from "node:url";
import * as path from "path";
import {
  CommitUpdater,
  deleteFolderIfExists,
} from "./functions/helper.js";
import { getDB } from "./db/index.js";
import { submissionTable, userTable } from "./db/schema.js";
import { eq } from "drizzle-orm";
import { promisify } from "util";
import {exec as execCallback} from 'child_process';
import { parseBenchmark, type BenchmarkStats } from "./functions/benchmark.js";

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
            return;
          }

          const submission = submissions[0];

          console.log(" [x] Inserted submission to db");

          const commitUpdater = new CommitUpdater(octokit, db, repository, after, submission);

          commitUpdater.run("pending", `Holon buddy, ${process.env.WORKER_NAME} here! Lemme cook...`).then(() => {
            console.log(" [x] Updated initial commit status for that moron");
          });

          const containerName = `${repository.owner.name}_${repository.name}`.toLowerCase();

          const folderPath = path.join(
            dirname(currentDirectory),
            `src/${containerName}`
          );
          deleteFolderIfExists(folderPath);

          console.log(" [*] Initiating clone and build");

          const { data: { token: installationToken } } = await octokit.request(`POST /app/installations/${installation.id}/access_tokens`, {
              installation_id: installation.id,
              repositories: [
                  repository.name
              ],
              permissions: {
                  contents: 'read'
              },
              headers: {
                  'X-GitHub-Api-Version': '2022-11-28'
              }
          })

          console.log(` [x] Got installation token (IDK for what): ${installationToken}`);

          const cloneUrl = repository.clone_url.replace(
            "https://",
            `https://x-access-token:${installationToken}@`
          ) as string;

          try {
              console.log(` [x] Git clone ${cloneUrl}`);
              const { stdout: cloneOutput } = await exec(`git clone ${cloneUrl} ${folderPath}`);
              console.log(` [x] Git clone output -> ${cloneOutput}`);

              await commitUpdater.run("pending", `Got your code, building...`);

              const { stdout: buildOutput } = await exec(`cd ${folderPath} && docker build -q -t ${containerName} .`);
              console.log(` [x] Docker build -> ${buildOutput}`);

              await commitUpdater.run("pending", `Segssy, build succeeded, running benchmarks...`);

          } catch (error: any) {
            await commitUpdater.run("error", "Clone or build fked up: " + error.toString());
            throw new Error(`Clone or build fked up: ${error}`);
          }

          let benchData: {
            parsed: BenchmarkStats;
            raw: Record<any, any>;
          };

          try {
            console.log(" [x] Build succeeded...")

            // TODO: make sure the submission container dockerfile installs pyperf inside the container
            
            console.log(" [x] Running benchmarks...");
            await exec(`docker run -d --name temp_${containerName} ${containerName} /venv/bin/python -m pyperf command -o /app/bench.json -p 1 -- python /app/script.py`);

            console.log(" [x] Fetching benchmark results...");
            await exec(`docker cp temp_${containerName}:/app/bench.json ./bench.json`);

            benchData = await parseBenchmark("./bench.json");
            console.log(" [x] Parsed benchmark Data: ", benchData);

            console.log(" [x] Stopping & removing container...");
            await exec(`docker rm -f temp_${containerName}`);
              
          } catch (error: any) {
            await commitUpdater.run("error", "Test and benchmark fked up: " + error.toString());
            throw new Error(`Test and benchmark fked up: ${error}`);
          }

          if (!benchData) {
            await commitUpdater.run("error", "That mf benchmark data ran away somewhere");
            throw new Error("That mf benchmark data ran away somewhere");
          }

          const {parsed: parsed_bench, raw: raw_bench} = benchData;

          await db.update(submissionTable).set({
            commit_status: `success | IDK how, but the whole thing workd, runtime: ${parsed_bench.mean} ms.`,
            runtime: parsed_bench.mean.toString(),
            parsed_json: parsed_bench,
            raw_json: raw_bench,
          }).where(eq(submissionTable.id, submission?.id || ""))

          await octokit.rest.repos.createCommitStatus({
            owner: repository.owner.login,
            repo: repository.name,
            sha: after,
            state: "success",
            description: `IDK how, but the whole thing workd, runtime: ${parsed_bench.mean} ms.`,
          });

          return;
          
        } catch (error) {
          console.error(" [-] Some shit went wrong: %s", error);
          return;
        }
      },
      {
        noAck: true,
      }
    );
  });
});