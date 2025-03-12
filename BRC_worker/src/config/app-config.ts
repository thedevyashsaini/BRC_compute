import { dirname } from "path";
import { fileURLToPath } from "node:url";
import { App } from "octokit";

export const QUEUE_NAME = "proposal";
export const RABBITMQ_URL = "amqp://rabbitmq";
export const WORKER_NAME = process.env.WORKER_NAME;
export const TEST_LEVEL = process.env.TEST_LEVEL;
export const UPGRADE_LEVEL = process.env.UPGRADE_LEVEL;
export const DATABASE_POOLER_URL = process.env.DATABASE_POOLER_URL;
export const BASE_DIR = dirname(dirname(fileURLToPath(import.meta.url)));

export function createGitHubApp() {
  const privateKey = Buffer.from(
    process.env.GITHUB_PRIVATE_KEY!,
    "base64"
  ).toString("utf8");

  return new App({
    appId: process.env.GITHUB_APP_ID!,
    privateKey: privateKey,
  });
}

export const BRC_INSTALLATION_ID = 61221514;
