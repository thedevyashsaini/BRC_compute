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
