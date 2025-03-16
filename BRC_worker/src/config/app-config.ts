import { dirname } from "path";
import { fileURLToPath } from "node:url";
import { App } from "octokit";

export const QUEUE_NAME = process.env.QUEUE_NAME || "proposal";
export const RABBITMQ_URL = process.env.RABBITMQ_URL || "amqp://rabbitmq";
export const WORKER_NAME = process.env.WORKER_NAME;
export const TEST_LEVEL = process.env.TEST_LEVEL;
export const UPGRADE_LEVEL = process.env.UPGRADE_LEVEL;
export const DATABASE_POOLER_URL = process.env.DATABASE_POOLER_URL;
export const BASE_DIR = dirname(dirname(fileURLToPath(import.meta.url)));

export function createGitHubApp() {
  const privateKey = process.env.GITHUB_PRIVATE_KEY!.split(String.raw`\n`).join('\n');

  console.log(privateKey);
    
  return new App({
    appId: process.env.GITHUB_APP_ID!,
    privateKey,
  });
}
