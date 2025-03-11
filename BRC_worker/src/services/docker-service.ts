import { promisify } from "util";
import { exec as execCallback } from "child_process";
import * as path from "path";
import { BASE_DIR } from "../config/app-config.js";

const exec = promisify(execCallback);

export class DockerService {
  async buildImage(containerName: string, folderPath: string): Promise<void> {
    console.log(" [*] Starting Docker build...");
    const { stdout, stderr } = await exec(
      `cd ${folderPath} && docker build -t ${containerName} .`
    );

    console.log(` [x] Docker build stdout -> ${stdout}`);
    if (stderr) {
      console.error(` [-] Docker build stderr -> ${stderr}`);
    }
    console.log(" [x] Docker build completed");
  }

  async runBenchmarks(
    containerName: string,
    folderPath: string,
    level: string
  ): Promise<void> {
    await exec(
      `cd ${folderPath} && LEVEL=${level} CONTAINER_NAME=${containerName} docker-compose up -d`
    );

    const { stdout: containerStatus } = await exec(
      `cd ${folderPath} && CONTAINER_NAME=${containerName} docker-compose ps --status running -q`
    );

    if (!containerStatus.trim()) {
      const { stdout: errorLogs } = await exec(
        `cd ${folderPath} && CONTAINER_NAME=${containerName} docker-compose logs`
      );
      console.error(" [-] Container logs:", errorLogs);
      throw new Error("Container failed to start or exited immediately");
    }

    const { stdout, stderr } = await exec(
      `cd ${folderPath} && CONTAINER_NAME=${containerName} docker-compose logs -f`
    );

    console.log(" [x] Benchmark logs:");
    console.log(stdout);

    if (stderr) {
      console.error(" [-] Benchmark stderr:", stderr);
    }

    await this.checkContainerExit(folderPath, containerName);
  }

  async checkContainerExit(
    folderPath: string,
    containerName: string
  ): Promise<void> {
    const { stdout: containerIds } = await exec(
      `cd ${folderPath} && CONTAINER_NAME=${containerName} docker-compose ps -q`
    );

    if (!containerIds.trim()) {
      const { stdout: previousContainers } = await exec(
        `docker ps -a --filter "name=${containerName}" --format "{{.ID}}"`
      );

      if (previousContainers.trim()) {
        const containerId = previousContainers.trim().split("\n")[0];
        const { stdout: exitCode } = await exec(
          `docker inspect -f '{{.State.ExitCode}}' ${containerId}`
        );

        if (exitCode.trim() && parseInt(exitCode.trim()) !== 0) {
          throw new Error(`Benchmark failed with exit code ${exitCode.trim()}`);
        }
      }
    } else {
      const { stdout: exitCode } = await exec(
        `cd ${folderPath} && CONTAINER_NAME=${containerName} docker-compose ps -q | xargs docker inspect -f '{{.State.ExitCode}}'`
      );

      if (exitCode.trim() && parseInt(exitCode.trim()) !== 0) {
        throw new Error(`Benchmark failed with exit code ${exitCode.trim()}`);
      }
    }
  }

  async copyOutputFromContainer(
    containerName: string,
    folderPath: string
  ): Promise<string> {
    const outputPath = path.join(folderPath, "output");
    await exec(`mkdir -p ${outputPath}`);

    const { stdout: containerId } = await exec(
      `cd ${folderPath} && CONTAINER_NAME=${containerName} docker-compose ps -q`
    );

    let containerIdentifier;

    if (!containerId.trim()) {
      const { stdout: containers } = await exec(
        `docker ps -a --filter "name=${containerName}" --format "{{.Names}}"`
      );
      containerIdentifier = containers.trim().split("\n")[0];

      if (!containerIdentifier) {
        throw new Error("Unable to find container to copy from");
      }
    } else {
      containerIdentifier = containerId.trim();
    }

    await exec(
      `docker cp ${containerIdentifier}:/usr/src/app/output/. ${outputPath}/`
    );

    const { stdout: lsOutput } = await exec(`ls -la ${outputPath}`);
    console.log(` [x] Output directory contents:`, lsOutput);

    return outputPath;
  }

  async cleanup(containerName: string): Promise<void> {
    try {
      await exec(`docker rm -f temp_${containerName}`);
    } catch (err) {
      console.error(`Failed to remove container: ${err}`);
    }
  }
}
