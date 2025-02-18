import amqp from "amqplib/callback_api.js";
import { App } from "octokit";
import { dirname } from "path";
import { fileURLToPath } from "node:url";
import * as path from 'path';
import {deleteFolderIfExists } from "./functions/helper.js";
import { getDB } from "./db/index.js";
import { submissionTable, userTable } from "./db/schema.js";
import { userInfo } from "node:os";
import { eq } from "drizzle-orm";

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


amqp.connect("amqp://localhost", function (error0, connection) {
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
          const {from, data} = JSON.parse(msg?.content.toString() || '{}');
          console.log(" [x] Received task from %s", from);
          console.log(" [x] Received task: %s", data.toString());

          if (!from || !data) {
            console.error(" [-] Invalid task");
            return;
          }

          const { ref, repository, installation, after } = data;

          const octokit = await githubApp.getInstallationOctokit(installation.id);

          console.log(" [x] Got octokit");

          const user = await db.select({
            id: userTable.id,
            email: userTable.email,
          }).from(userTable).where(eq(userTable.github_repo, repository.id));

          if (!user || user.length === 0) {
            console.error(" [-] Fking user not found");
            return;
          }

          console.log(" [x] Got repo owner from db");
          
          const submissions = await db.insert(submissionTable).values({
            user_id: user[0]?.id || "",
            commit_hash: after,
            commit_status: "Initializing",
          }).returning();

          if (!submissions || submissions.length === 0) {
            console.error(" [-] Fking submission not found");
            return;
          }

          const submission = submissions[0];

          console.log(" [x] Inserted submission to db");

          await octokit.rest.repos.createCommitStatus({
            owner: repository.owner.login,
            repo: repository.name,
            sha: after,
            state: "pending",
            description: `Holon buddy, ${process.env.worker_name} here! Lemme cook...`,
          });

          console.log(" [x] Updated commit status for that moron");

          const repoUrl = repository.clone_url;
          const folderPath = path.join(dirname(currentDirectory), `src/${repository.owner.name}${repository.name}`);
          deleteFolderIfExists(folderPath);

        } catch (error) {
          console.error(" [-] Some shit went wrong: %s", error);
          return
        }
      },
      {
        noAck: true,
      }
    );

  });
});