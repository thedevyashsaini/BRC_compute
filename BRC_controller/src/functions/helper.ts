// Function to delete the folder if it exists
import * as fs from "node:fs";
import { Webhooks } from "@octokit/webhooks";
import * as path from "path";
import {type Request, type Response} from "express";
import dotenv from "dotenv";
import {exec} from "node:child_process";

dotenv.config();

export function deleteFolderIfExists(folderPath: string) {
    if (fs.existsSync(folderPath)) {
        fs.rmdirSync(folderPath, { recursive: true });
        console.log(`Folder '${folderPath}' deleted successfully.`);
    } else {
        console.log(`Folder '${folderPath}' does not exist.`);
    }
};

export function cleanseString(input: string) {
    return input
        .replace(/\s+/g, '')       // Remove all whitespaces and new lines
        .replace(/[^a-zA-Z0-9_]/g, ''); // Remove anything that's not alphanumeric or an underscore
}

export const handleWebhook = async (req: Request, res: Response) => {
    const webhooks = new Webhooks({
        secret: process.env.GITHUB_WEBHOOK_SECRET!,
    });


    const signature = req.headers["x-hub-signature-256"] as string;
    //@ts-ignore
    const body = req.rawBody;

    if (!(await webhooks.verify(body, signature))) {
        res.status(401).send("Unauthorized");
        return false;
    }

    return true
};

function cloneRepo(repositoryUrl: string, token: string) {
    const authenticatedUrl = repositoryUrl.replace(
        "https://",
        `https://${token}@`
    );

    exec(`git clone ${authenticatedUrl}`, (error, stdout, stderr) => {
        if (error) {
            console.error(`Error cloning repository: ${error.message}`);
            return;
        }
        console.log(`Repository cloned successfully: ${stdout}`);
    });
}

export const validateRepoStructure = async (repoPath: string): Promise<boolean> => {
    const requiredFiles = [
        '.dockerignore',
        '.gitignore',
        'Dockerfile',
        'docker-compose.yml',
        'requirements.txt',
        'src/app/main.py',
        'src/app/yourbot/bot.py',
        'src/app/yourbot/main.py',
        'src/test/actually_dumbbot.py',
        'src/test/test.py',
        'src/test/test_bot.py',
        'src/test/test_dumbbot.py'
    ];

    try {

        const directories = [
            '.github/workflows',
            'src/app/yourbot',
            'src/test'
        ];

        for (const dir of directories) {
            const dirPath = path.join(repoPath, dir);
            if (!fs.existsSync(dirPath)) {
                console.log(`Missing required directory: ${dir}`);
                return false;
            }
        }

        for (const file of requiredFiles) {
            const filePath = path.join(repoPath, file);
            if (!fs.existsSync(filePath)) {
                console.log(`Missing required file: ${file}`);
                return false;
            }
        }

        return true;
    } catch (error) {
        console.error('Error validating repository structure:', error);
        return false;
    }
};