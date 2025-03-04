import * as fs from "fs/promises";
import { DateTime } from "luxon";

export interface BenchmarkStats {
  total_duration: number;
  start_date: DateTime;
  end_date: DateTime;
  raw_min: number;
  raw_max: number;
  calibration_runs: number;
  value_runs: number;
  total_runs: number;
  warmups_per_run: number;
  values_per_run: number;
  loop_iterations: number;
  total_values: number;
  minimum: number;
  median: number;
  mad: number;
  mean: number;
  stddev: number;
  maximum: number;
  percentiles: Record<string, number>;
  outliers: number;
}

function median(values: number[]): number { 
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  if (sorted.length === 0) {
    throw new Error("Cannot calculate median of an empty array");
  }
  return (
    (sorted.length % 2 !== 0
      ? sorted[mid]
      : (sorted[mid - 1]! + sorted[mid]!) / 2) || 1000
  );
}

function mean(values: number[]): number {
  return values.reduce((a, b) => a + b) / values.length;
}

function stddev(values: number[]): number {
  const m = mean(values);
  const variance =
    values.reduce((a, b) => a + Math.pow(b - m, 2), 0) / (values.length - 1);
  return Math.sqrt(variance);
}

function percentile(values: number[], p: number): number {
  if (values.length === 0) {
    throw new Error("Cannot calculate percentile of an empty array");
  }

  const sorted = [...values].sort((a, b) => a - b);
  const pos = (sorted.length - 1) * p;
  const base = Math.floor(pos);
  const rest = pos - base;

  if (base === sorted.length - 1 || sorted[base + 1] === undefined) {
    return sorted[base]!;
  }

  return sorted[base]! + rest * (sorted[base + 1]! - sorted[base]!);
}

export async function parseBenchmark(
  filename: string
): Promise<{ parsed: BenchmarkStats; raw: Record<any, any> }> {
  const content = await fs.readFile(filename, "utf-8");
  const data = JSON.parse(content);
  const benchmark = data.benchmarks[0];
  const runs = benchmark.runs;

  const allValues: number[] = [];
  for (const run of runs.slice(1)) {
    allValues.push(...run.values);
  }

  const valuesMs = allValues.map((v) => v * 1000000);

  const firstRun = runs[0].metadata.date;
  const lastRun = runs[runs.length - 1].metadata.date;
  const startDate = DateTime.fromFormat(firstRun, "yyyy-MM-dd HH:mm:ss.SSS");
  const endDate = DateTime.fromFormat(lastRun, "yyyy-MM-dd HH:mm:ss.SSS");
  const totalDuration = endDate.diff(startDate).as("seconds");

  const medianValue = median(valuesMs);
  const deviations = valuesMs.map((x) => Math.abs(x - medianValue));
  const mad = median(deviations);
  const meanValue = mean(valuesMs);
  const stddevValue = stddev(valuesMs);

  const percentiles: Record<string, number> = {
    "0th": percentile(valuesMs, 0),
    "5th": percentile(valuesMs, 0.05),
    "25th": percentile(valuesMs, 0.25),
    "50th": percentile(valuesMs, 0.5),
    "75th": percentile(valuesMs, 0.75),
    "95th": percentile(valuesMs, 0.95),
    "100th": percentile(valuesMs, 1),
  };

  const q1 = percentile(valuesMs, 0.25);
  const q3 = percentile(valuesMs, 0.75);
  const iqr = q3 - q1;
  const lowerBound = q1 - 1.5 * iqr;
  const upperBound = q3 + 1.5 * iqr;
  const outliers = valuesMs.filter(
    (x) => x < lowerBound || x > upperBound
  ).length;

  return {
    parsed: {
      total_duration: totalDuration,
      start_date: startDate,
      end_date: endDate,
      raw_min: Math.min(...allValues) * 1000,
      raw_max: Math.max(...allValues) * 1000,
      calibration_runs: 1,
      value_runs: runs.length - 1,
      total_runs: runs.length,
      warmups_per_run: 1,
      values_per_run: runs[1].values.length,
      loop_iterations: 128,
      total_values: valuesMs.length,
      minimum: Math.min(...valuesMs),
      median: medianValue,
      mad: mad,
      mean: meanValue,
      stddev: stddevValue,
      maximum: Math.max(...valuesMs),
      percentiles: percentiles,
      outliers: outliers,
    },
    raw: data,
  };
}
