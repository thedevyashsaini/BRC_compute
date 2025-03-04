use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::collections::HashMap;
use std::fs;

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

#[derive(Debug, Deserialize)]
struct Run {
    values: Vec<f64>,
    metadata: Metadata,
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
    let variance = values
        .iter()
        .map(|x| (x - m).powi(2))
        .sum::<f64>() / (values.len() - 1) as f64;
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

pub fn parse(filename: &str) -> Result<(BenchmarkStats, Value)> {
    let content = fs::read_to_string(filename).expect("Failed to read file");
    let data: Value = serde_json::from_str(&content)?;
    let benchmark = &data["benchmarks"][0];
    let runs: Vec<Run> = serde_json::from_value(benchmark["runs"].clone())?;

    let mut all_values = Vec::new();
    for run in runs.iter().skip(1) {
        all_values.extend(&run.values);
    }

    let values_ms: Vec<f64> = all_values.iter().map(|&v| v * 1_000_000.0).collect();

    let first_run = &runs[0].metadata.date;
    let last_run = &runs[runs.len() - 1].metadata.date;
    
    let start_date = NaiveDateTime::parse_from_str(first_run, "%Y-%m-%d %H:%M:%S%.3f")
        .unwrap()
        .and_utc();
    let end_date = NaiveDateTime::parse_from_str(last_run, "%Y-%m-%d %H:%M:%S%.3f")
        .unwrap()
        .and_utc();
    let total_duration = (end_date - start_date).num_seconds() as f64;

    let median_value = median(&values_ms);
    let deviations: Vec<f64> = values_ms.iter().map(|&x| (x - median_value).abs()).collect();
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

    let stats = BenchmarkStats {
        total_duration,
        start_date,
        end_date,
        raw_min: all_values.iter().copied().fold(f64::INFINITY, f64::min) * 1000.0,
        raw_max: all_values.iter().copied().fold(f64::NEG_INFINITY, f64::max) * 1000.0,
        calibration_runs: 1,
        value_runs: (runs.len() - 1) as u32,
        total_runs: runs.len() as u32,
        warmups_per_run: 1,
        values_per_run: runs[1].values.len(),
        loop_iterations: 128,
        total_values: values_ms.len(),
        minimum: *values_ms.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
        median: median_value,
        mad,
        mean: mean_value,
        stddev: stddev_value,
        maximum: *values_ms.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
        percentiles,
        outliers,
    };

    Ok((stats, data))
}