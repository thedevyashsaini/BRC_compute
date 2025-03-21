use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct BenchmarkStats {
    total_duration: f64,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    raw_min: f64,
    raw_max: f64,
    calibration_runs: u32,
    value_runs: u32,
    total_runs: u32,
    warmups_per_run: u32,
    values_per_run: usize,
    loop_iterations: u32,
    total_values: usize,
    minimum: f64,
    median: f64,
    mad: f64,
    mean: f64,
    stddev: f64,
    maximum: f64,
    percentiles: HashMap<String, f64>,
    outliers: usize,
}

impl BenchmarkStats {
    pub fn get_mean(&self) -> f64 {
        self.mean
    }
}

#[derive(Debug, Deserialize)]
struct Run {
    values: Option<Vec<f64>>,
    warmups: Vec<(u32, f64)>,
    metadata: Option<Metadata>, // Make metadata optional
}

#[derive(Debug, Deserialize)]
struct Metadata {
    date: String,
}

fn median(values: &[f64]) -> f64 {
    if values.is_empty() {
        panic!("Cannot calculate median of an empty array");
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = sorted.len() / 2;

    if sorted.len() % 2 != 0 {
        sorted[mid]
    } else {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    }
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn stddev(values: &[f64]) -> f64 {
    let m = mean(values);
    let variance = values.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    variance.sqrt()
}

fn percentile(values: &[f64], p: f64) -> f64 {
    if values.is_empty() {
        panic!("Cannot calculate percentile of an empty array");
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let pos = (sorted.len() - 1) as f64 * p;
    let base = pos.floor() as usize;
    let rest = pos - base as f64;

    if base == sorted.len() - 1 {
        return sorted[base];
    }

    sorted[base] + rest * (sorted[base + 1] - sorted[base])
}

pub fn parse(
    data: serde_json::Value,
    skipped_calibration: bool,
) -> Result<(BenchmarkStats, Value)> {
    let benchmark: &Value = &data["benchmarks"][0];
    let runs: Vec<Run> = serde_json::from_value(benchmark["runs"].clone())?;

    let mut all_values: Vec<f64> = Vec::new();

    // Handle differently based on whether calibration was skipped
    if skipped_calibration {
        // When calibration is skipped, use all runs
        for run in &runs {
            if let Some(values) = &run.values {
                all_values.extend(values);
            }
        }
    } else {
        // When calibration is included, skip the first run (calibration run)
        for run in runs.iter().skip(1) {
            if let Some(values) = &run.values {
                all_values.extend(values);
            }
        }
    }

    let values_ms: Vec<f64> = all_values.iter().map(|&v| v * 1_000_000.0).collect();

    // Handle dates differently based on whether calibration was skipped
    let (start_date, end_date) = if skipped_calibration {
        // When calibration is skipped, use metadata from the top-level object
        let date_str = data["metadata"]["date"]
            .as_str()
            .unwrap_or("2025-03-21 08:00:00.000");
        let date = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S%.3f")
            .unwrap_or_else(|_| {
                NaiveDateTime::parse_from_str("2025-03-21 08:00:00.000", "%Y-%m-%d %H:%M:%S%.3f")
                    .unwrap()
            })
            .and_utc();
        // For end date, add the duration from metadata if available
        let duration = data["metadata"]["duration"].as_f64().unwrap_or(0.0);
        let end = date + chrono::Duration::seconds(duration as i64);
        (date, end)
    } else {
        // Normal case: use metadata from first and last run if available
        let first_date = runs
            .first()
            .and_then(|run| run.metadata.as_ref())
            .map(|meta| meta.date.as_str())
            .unwrap_or("2025-03-21 08:00:00.000");

        let last_date = runs
            .last()
            .and_then(|run| run.metadata.as_ref())
            .map(|meta| meta.date.as_str())
            .unwrap_or_else(|| {
                "2025-03-21 08:00:00.000"
            });
        let start = NaiveDateTime::parse_from_str(first_date, "%Y-%m-%d %H:%M:%S%.3f")
            .unwrap_or_else(|_| {
                NaiveDateTime::parse_from_str("2025-03-21 08:00:00.000", "%Y-%m-%d %H:%M:%S%.3f")
                    .unwrap()
            })
            .and_utc();
        let end = NaiveDateTime::parse_from_str(last_date, "%Y-%m-%d %H:%M:%S%.3f")
            .unwrap_or_else(|_| {
                NaiveDateTime::parse_from_str("2025-03-21 08:00:00.000", "%Y-%m-%d %H:%M:%S%.3f")
                    .unwrap()
            })
            .and_utc();
        (start, end)
    };

    let total_duration = (end_date - start_date).num_seconds() as f64;

    let median_value = median(&values_ms);
    let deviations: Vec<f64> = values_ms
        .iter()
        .map(|&x| (x - median_value).abs())
        .collect();
    let mad = median(&deviations);
    let mean_value = mean(&values_ms);
    let stddev_value = stddev(&values_ms);

    let mut percentiles = HashMap::new();
    percentiles.insert("0th".to_string(), percentile(&values_ms, 0.0));
    percentiles.insert("5th".to_string(), percentile(&values_ms, 0.05));
    percentiles.insert("25th".to_string(), percentile(&values_ms, 0.25));
    percentiles.insert("50th".to_string(), percentile(&values_ms, 0.5));
    percentiles.insert("75th".to_string(), percentile(&values_ms, 0.75));
    percentiles.insert("95th".to_string(), percentile(&values_ms, 0.95));
    percentiles.insert("100th".to_string(), percentile(&values_ms, 1.0));

    let q1 = percentile(&values_ms, 0.25);
    let q3 = percentile(&values_ms, 0.75);
    let iqr = q3 - q1;
    let lower_bound = q1 - 1.5 * iqr;
    let upper_bound = q3 + 1.5 * iqr;
    let outliers = values_ms
        .iter()
        .filter(|&&x| x < lower_bound || x > upper_bound)
        .count();

    // Get loop iterations - handling both cases safely
    let loop_iterations = if !runs.is_empty() && !runs[0].warmups.is_empty() {
        runs[0].warmups[0].0 as u32
    } else {
        // Fallback to metadata or default value
        data["metadata"]["loops"].as_u64().unwrap_or(1) as u32
    };

    let warmups_per_run = if skipped_calibration {
        runs.first().map(|r| r.warmups.len() as u32).unwrap_or(0)
    } else {
        runs.iter()
            .skip(1)
            .next()
            .map(|r| r.warmups.len() as u32)
            .unwrap_or(0)
    };

    let values_per_run = if skipped_calibration {
        runs.first()
            .and_then(|r| r.values.as_ref().map(|v| v.len()))
            .unwrap_or(0)
    } else {
        runs.iter()
            .skip(1)
            .next()
            .and_then(|r| r.values.as_ref().map(|v| v.len()))
            .unwrap_or(0)
    };

    let stats = BenchmarkStats {
        total_duration,
        start_date,
        end_date,
        // Convert to microseconds
        raw_min: all_values.iter().copied().fold(f64::INFINITY, f64::min) * 1_000_000.0,
        raw_max: all_values.iter().copied().fold(f64::NEG_INFINITY, f64::max) * 1_000_000.0,
        calibration_runs: if skipped_calibration {
            0
        } else {
            runs.first().map(|r| r.warmups.len() as u32).unwrap_or(0)
        },
        value_runs: if skipped_calibration {
            runs.len() as u32
        } else {
            (runs.len() - 1) as u32
        },
        total_runs: runs.len() as u32,
        warmups_per_run,
        values_per_run,
        loop_iterations,
        total_values: values_ms.len(),
        minimum: *values_ms
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(&0.0),
        median: median_value,
        mad,
        mean: mean_value,
        stddev: stddev_value,
        maximum: *values_ms
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(&0.0),
        percentiles,
        outliers,
    };

    Ok((stats, data))
}
