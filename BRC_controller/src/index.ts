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

const app = express();

app.use(
  bodyParser.json({
    verify: (req: any, res, buf) => {
      req.rawBody = buf.toString();
    },
  })
);

const privateKey = Buffer.from(
  process.env.GITHUB_PRIVATE_KEY!,
  "base64"
).toString("utf8");

const githubApp = new App({
  appId: process.env.GITHUB_APP_ID!,
  privateKey: privateKey,
});

app.get("/", (req: Request, res: Response) => {
  res.send("FK OFF YOU ASSHOLE");
});

app.get("/test", async (req: Request, res: Response) => {
  try {
    amqp.connect("amqp://rabbitmq", function (error0: any, connection: any) {
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
          from: "controller",
          data: {
            ref: "refs/heads/master",
            repository: {
              id: 945106473,
              node_id: "R_kgDOOFUuKQ",
              name: "BRC_MySubmission",
              full_name: "thedevyashsaini/BRC_MySubmission",
              private: true,
              owner: {
                name: "thedevyashsaini",
                email: "59441567+thedevyashsaini@users.noreply.github.com",
                login: "thedevyashsaini",
                id: 59441567,
                node_id: "MDQ6VXNlcjU5NDQxNTY3",
                avatar_url:
                  "https://avatars.githubusercontent.com/u/59441567?v=4",
                gravatar_id: "",
                url: "https://api.github.com/users/thedevyashsaini",
                html_url: "https://github.com/thedevyashsaini",
                followers_url:
                  "https://api.github.com/users/thedevyashsaini/followers",
                following_url:
                  "https://api.github.com/users/thedevyashsaini/following{/other_user}",
                gists_url:
                  "https://api.github.com/users/thedevyashsaini/gists{/gist_id}",
                starred_url:
                  "https://api.github.com/users/thedevyashsaini/starred{/owner}{/repo}",
                subscriptions_url:
                  "https://api.github.com/users/thedevyashsaini/subscriptions",
                organizations_url:
                  "https://api.github.com/users/thedevyashsaini/orgs",
                repos_url: "https://api.github.com/users/thedevyashsaini/repos",
                events_url:
                  "https://api.github.com/users/thedevyashsaini/events{/privacy}",
                received_events_url:
                  "https://api.github.com/users/thedevyashsaini/received_events",
                type: "User",
                user_view_type: "public",
                site_admin: false,
              },
              html_url: "https://github.com/thedevyashsaini/BRC_MySubmission",
              description: null,
              fork: false,
              url: "https://github.com/thedevyashsaini/BRC_MySubmission",
              forks_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/forks",
              keys_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/keys{/key_id}",
              collaborators_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/collaborators{/collaborator}",
              teams_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/teams",
              hooks_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/hooks",
              issue_events_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/issues/events{/number}",
              events_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/events",
              assignees_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/assignees{/user}",
              branches_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/branches{/branch}",
              tags_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/tags",
              blobs_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/git/blobs{/sha}",
              git_tags_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/git/tags{/sha}",
              git_refs_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/git/refs{/sha}",
              trees_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/git/trees{/sha}",
              statuses_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/statuses/{sha}",
              languages_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/languages",
              stargazers_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/stargazers",
              contributors_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/contributors",
              subscribers_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/subscribers",
              subscription_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/subscription",
              commits_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/commits{/sha}",
              git_commits_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/git/commits{/sha}",
              comments_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/comments{/number}",
              issue_comment_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/issues/comments{/number}",
              contents_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/contents/{+path}",
              compare_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/compare/{base}...{head}",
              merges_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/merges",
              archive_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/{archive_format}{/ref}",
              downloads_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/downloads",
              issues_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/issues{/number}",
              pulls_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/pulls{/number}",
              milestones_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/milestones{/number}",
              notifications_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/notifications{?since,all,participating}",
              labels_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/labels{/name}",
              releases_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/releases{/id}",
              deployments_url:
                "https://api.github.com/repos/thedevyashsaini/BRC_MySubmission/deployments",
              created_at: 1741453994,
              updated_at: "2025-03-09T14:01:05Z",
              pushed_at: 1741529078,
              git_url: "git://github.com/thedevyashsaini/BRC_MySubmission.git",
              ssh_url: "git@github.com:thedevyashsaini/BRC_MySubmission.git",
              clone_url:
                "https://github.com/thedevyashsaini/BRC_MySubmission.git",
              svn_url: "https://github.com/thedevyashsaini/BRC_MySubmission",
              homepage: null,
              size: 9922,
              stargazers_count: 0,
              watchers_count: 0,
              language: "Python",
              has_issues: true,
              has_projects: true,
              has_downloads: true,
              has_wiki: true,
              has_pages: false,
              has_discussions: false,
              forks_count: 0,
              mirror_url: null,
              archived: false,
              disabled: false,
              open_issues_count: 0,
              license: null,
              allow_forking: true,
              is_template: false,
              web_commit_signoff_required: false,
              topics: [],
              visibility: "private",
              forks: 0,
              open_issues: 0,
              watchers: 0,
              default_branch: "master",
              stargazers: 0,
              master_branch: "master",
            },
            installation: {
              id: 61221514,
              node_id: "MDIzOkludGVncmF0aW9uSW5zdGFsbGF0aW9uNjEyMjE1MTQ=",
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

    amqp.connect("amqp://rabbitmq", function (error0: any, connection: any) {
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

        channel.sendToQueue(
          queue,
          Buffer.from(
            JSON.stringify({
              from: "controller",
              data: { ref, repository, installation, after },
            })
          )
        );
        console.log(
          " [x] Sent %s",
          JSON.stringify({ ref, repository, installation, after })
        );
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
