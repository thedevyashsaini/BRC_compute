import amqp from "amqplib/callback_api";
import bodyParser from "body-parser";
import express, { type Request, type Response } from "express";
import {App} from "octokit";
import { promisify } from "util";
import { exec as execCallback } from "child_process";
import { dirname } from "path";
import { fileURLToPath } from "node:url";
import dotenv from "dotenv";
import {
  handleWebhook,
} from "./functions/helper";
import { getDB } from "./db/index.js";
import { eq } from "drizzle-orm";
import { userTable } from "./db/schema";

const exec = promisify(execCallback);
process.chdir(dirname(fileURLToPath(import.meta.url)));

dotenv.config({path: ".env.local"});

const app = express();

app.use(
  bodyParser.json({
    verify: (req: any, res, buf) => {
      req.rawBody = buf.toString();
    },
  })
);

const privateKey = Buffer.from(process.env.GITHUB_PRIVATE_KEY!, "base64").toString(
  "utf8"
);

const githubApp = new App({
  appId: process.env.GITHUB_APP_ID!,
  privateKey: privateKey,
});

app.get("/", (req: Request, res: Response) => {
  res.send("FK OFF YOU ASSHOLE");
});

app.post("/commit", async (req: Request, res: Response): Promise<void> => {
  try {
    try {
      if (!(await handleWebhook(req, res))) {
        res.status(400).send("Secrets don't match homeboy.");
        console.error(" [-] Secrets didn't match.");
        return;
      }
    } catch (e) {
      res.status(400).send("Secrets don't match homeboy.");
      console.error(" [-] ", e);
      return;
    }

    const db = await getDB();
    const { ref, repository, installation, after } = req.body;
    console.log(
      " [x] Got request - ",
      repository,
      repository.master_branch,
      ref,
      after
    );

    if (ref !== `refs/heads/${repository.master_branch}`) {
      res.status(200).send("Not on main branch, skipping.");
      console.error(" [-] Not main branch.");
      return;
    }

    if (!repository || !repository.url) {
      res.status(200).send("Repository URL doesn't exist, it's smth else.");
      console.error(" [-] Repo URL doesn't exist.");
      return;
    }

    const repos = await db.query.userTable.findFirst({
      where: eq(userTable.github_repo, repository.id),
    });

    console.log(" [x] Got repos list: ", repos);

    if (!repos) {
      console.error(` [-] Not a tracked repository: ${repository.url}`);
      res.status(200).send("Not a tracked repository.");
      return;
    }

    const octokit = await githubApp.getInstallationOctokit(installation.id);

    await octokit.rest.repos.createCommitStatus({
      owner: repository.owner.login,
      repo: repository.name,
      sha: after,
      state: "pending",
      description: "Initial screen done, sending to test unit...",
    });

    amqp.connect("amqp://localhost", function (error0, connection) {
      if (error0) {
        throw error0;
      }
      connection.createChannel(function (error1, channel) {
        if (error1) {
          throw error1;
        }
        var queue = "tests";

        channel.assertQueue(queue, {
          durable: false,
        });

        channel.sendToQueue(queue, Buffer.from(JSON.stringify({ ref, repository, installation, after })));
        console.log(" [x] Sent %s", JSON.stringify({ ref, repository, installation, after }));
      });
    });

  } catch (error) {
    console.error("Unexpected error:", error);
    res.status(500).send("An unexpected error occurred");
  }
});

app.listen(5000, () => {
  console.log(`[server]: Server is running at http://localhost:${5000}`);
});