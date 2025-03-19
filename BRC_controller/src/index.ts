import amqp from "amqplib/callback_api.js";
import bodyParser from "body-parser";
import express, { type Request, type Response } from "express";
import { App } from "octokit";
import dotenv from "dotenv";
import { handleWebhook } from "./functions/helper.js";
import { getDB } from "./db/index.js";
import { eq } from "drizzle-orm";
import { userTable } from "./db/schema.js";

dotenv.config({ path: ".env" });

interface MessageRepo {
  name: string;
  id: string;
  clone_url: string;
  owner: {
    login: string;
  };
}

interface MessageData {
  repository: MessageRepo;
  installation: {
    id: number;
  };
  after: string;
}


const app = express();

app.use(
  bodyParser.json({
    verify: (req: any, res, buf) => {
      req.rawBody = buf.toString();
    },
  })
);

const privateKey = process.env.GITHUB_PRIVATE_KEY!.split(String.raw`\n`).join('\n')

const githubApp = new App({
  appId: process.env.GITHUB_APP_ID!,
  privateKey: privateKey,
});

app.get("/", (req: Request, res: Response) => {
  res.send("FK OFF YOU ASSHOLE");
});

app.get("/test", async (req: Request, res: Response) => {
  try {
    amqp.connect(process.env.RABBITMQ_URL || "amqp://rabbitmq", function (error0: any, connection: any) {
      if (error0) {
        throw error0;
      }
      connection.createChannel(function (error1: any, channel: any) {
        if (error1) {
          throw error1;
        }
        var queue = "proposal";

        channel.assertQueue(queue, {
          durable: false,
        });

        let data = {
          from: "push",
          data: {
            repository: {
              id: 945106473,
              name: "BRC_MySubmission",
              owner: {
                login: "thedevyashsaini",
              },
              clone_url: "https://github.com/thedevyashsaini/BRC_MySubmission.git",
            },
            installation: {
              id: 61221514,
            },
            after: "2154164af8c89b49627b741f67976e839b465fec",
          },
        };
        channel.sendToQueue(queue, Buffer.from(JSON.stringify(data)));
        console.log(" [x] Sent %s", JSON.stringify(data));
      });
    });
    res.status(200).send("Sent data to queue.");
  } catch (error) {
    console.error("Unexpected error:", error);
    res.status(500).send("An unexpected error occurred");
  }
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
    console.log(" [x] Got request - ", repository);

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

    amqp.connect(process.env.RABBITMQ_URL || "amqp://rabbitmq", function (error0: any, connection: any) {
      if (error0) {
        throw error0;
      }
      connection.createChannel(function (error1: any, channel: any) {
        if (error1) {
          throw error1;
        }
        var queue = "proposal";

        channel.assertQueue(queue, {
          durable: false,
        });

        const data: MessageData = {
          repository: {
            name: repository.name,
            id: repository.id,
            clone_url: repository.clone_url,
            owner: {
              login: repository.owner.login
            }
          },
          installation: {
            id: installation.id
          },
          after
        }

        channel.sendToQueue(
          queue,
          Buffer.from(
            JSON.stringify({
              from: "push",
              data
            })
          )
        );
        
        console.log(
          " [x] Sent %s",
          JSON.stringify(data)
        );
      });
    });
  } catch (error) {
    console.error("Unexpected error:", error);
    res.status(500).send("An unexpected error occurred");
  }
});

app.post("/upgrade", async (req: Request, res: Response): Promise<void> => {
  if (req.headers.authorization !== process.env.UPGRADE_SECRET) {
    res.status(401).send("Unauthorized");
    return;
  }

  const { owner, repo, commitHash } = req.body;

  if (!owner || !repo || !commitHash) {
    res.status(400).send("Missing required parameters: owner, repo, commitHash");
    return;
  }

  try {
    // Get the installation for the specified repository
    const {data: installation} = await githubApp.octokit.request('GET /repos/{owner}/{repo}/installation', {
      owner,
      repo,
      headers: {
        'X-GitHub-Api-Version': '2022-11-28'
      }
    });

    console.log(installation)

    // Get an authenticated octokit client for this installation
    const octokit = await githubApp.getInstallationOctokit(installation.id);

    const { data: repository } = await octokit.rest.repos.get({
      owner,
      repo
    })

    // Use the octokit client to create a commit status
    await octokit.rest.repos.createCommitStatus({
      owner,
      repo,
      sha: commitHash,
      state: "pending",
      description: "Processing upgrade request..."
    });
    repository.owner.name = owner;
    console.log(repository)

    // Send the upgrade request to the queue
    amqp.connect(process.env.RABBITMQ_URL || "amqp://rabbitmq", function (error0: any, connection: any) {
      if (error0) {
        throw error0;
        return;
      }
      connection.createChannel(function (error1: any, channel: any) {
        if (error1) {
          throw error1;
          return;
        }
        var queue = "divorce";

        channel.assertQueue(queue, {
          durable: false,
        });

        channel.sendToQueue(
          queue,
          Buffer.from(
            JSON.stringify({
              from: "upgrade",
              data: { repository, installation, after: commitHash },
            })
          )
        );
        console.log(" [x] Sent upgrade request for", owner, repo, commitHash);
      });
    });

    res.status(200).send("Upgrade request accepted");
  } catch (error: any) {
    console.error("Error processing upgrade request:", error);
    res.status(500).send(`Error: ${error.message || "Unknown error"}`);
  }
});

app.listen(5000, () => {
  console.log(`[server]: Server is running at http://localhost:${5000}`);
});
