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

interface BenchmarkMetadata {
  aslr: string;
  boot_time: string;
  command: string;
  cpu_config: string;
  cpu_count: number;
  cpu_freq: string;
  cpu_model_name: string;
  hostname: string;
  load_avg_1min: number;
  loops: number;
  name: string;
  perf_version: string;
  platform: string;
  runnable_threads: number;
  unit: string;
}

interface WarmupRun {
  0: number; // Number of iterations
  1: number; // Duration
}

interface RunMetadata {
  calibrate_loops?: number;
  command_max_rss: number;
  date: string;
  duration: number;
  uptime: number;
}

interface BenchmarkRun {
  metadata: RunMetadata;
  warmups: WarmupRun[];
  values?: number[]; // Only present in non-warmup runs
}

interface Benchmark {
  runs: BenchmarkRun[];
}

export interface BenchmarkRawResult {
  benchmarks: Benchmark[];
  metadata: BenchmarkMetadata;
  version: string;
}