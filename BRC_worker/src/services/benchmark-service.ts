import * as fs from "fs/promises";
import * as path from "path";
import { type BenchmarkStats } from "../models/benchmark-result.js";

export interface BenchmarkResult {
  parsed: BenchmarkStats;
  raw: Record<string, any>;
  status: {
    success: boolean;
    message: string;
  };
}

export class BenchmarkService {
  async extractResults(outputPath: string): Promise<BenchmarkResult> {
    const statusPath = path.join(outputPath, "status.json");
    const benchPath = path.join(outputPath, "bench.json");
    const benchParsedPath = path.join(outputPath, "bench_parsed.json");

    try {
      const statusContent = await fs.readFile(statusPath, "utf-8");
      const status = JSON.parse(statusContent);

      if (!status.success) {
        throw new Error(`Benchmark failed: ${status.message}`);
      }

      const benchContent = await fs.readFile(benchParsedPath, "utf-8");
      const bench = JSON.parse(benchContent);
      const benchParsedContent = await fs.readFile(benchParsedPath, "utf-8");
      const benchRaw = JSON.parse(benchParsedContent);

      return {
        status,
        parsed: benchRaw[0],
        raw: bench,
      };
    } catch (error) {
      throw new Error(`Failed to extract benchmark results: ${error}`);
    }
  }

  formatRuntimeDescription(mean: number): string {
    return `Runtime: ${Math.floor((mean / 1000) * 1000) / 1000} ms`;
  }
}
