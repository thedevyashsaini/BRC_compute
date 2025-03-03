use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::Path,
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};
use std::fmt::Debug;
use tokio::{
    sync::mpsc,
    task,
};

const CHUNK_SIZE: usize = 1_000_000;
const NUM_WORKERS: usize = 10;

pub async fn solve_testcase(input_file: &str) -> std::io::Result<()> {
    let file = File::open(input_file)?;
    let file_hash = input_file.split("_").last().unwrap().split(".").next().unwrap();
    let row_count = input_file.split("_").nth(1).unwrap().parse::<usize>().unwrap();
    println!("Solving test case file: {}", input_file);
    println!("File hash: {}", file_hash);
    println!("Row count: {}", row_count);

    let reader = BufReader::new(file);

    let output_file = format!("testcases/answer_{}_{}.txt", row_count,file_hash);
    let mut file = OpenOptions::new().create(true).write(true).truncate(true).open(output_file)?;


    let mut records: HashMap<String, (f32, f64, f32, usize)> = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        let (city, temp_str) = line.split_once(";").unwrap();
        let temp: f32 = temp_str.parse().unwrap();
        // println!("City: {}, Temp: {}", city, temp);
        let city_string = city.to_string();

        match records.get(&city_string) {
            Some(temps) => {
                let min_temp = if temp < temps.0 { temp } else { temps.0 };
                let total_temp = temps.1 + temp as f64;
                let max_temp = if temp > temps.2 { temp } else { temps.2 };
                let count = temps.3 + 1;
                records.insert(city_string, (min_temp, total_temp, max_temp, count));
            }
            None => {
                records.insert(city_string, (temp, temp as f64, temp, 1));
            }
        }
    }

    let mut keys: Vec<String> = records.keys().cloned().collect();
    keys.sort();
    
    for key in keys {
        let value = records.get(&key);

        match value {
            Some((min_temp, total_temp, max_temp, count)) => {
                let mut avg_temp = (((total_temp / *count as f64) * 10.0).round()) / 10.0;
                if avg_temp == 0.0 {
                    avg_temp = 0.0;
                }
                writeln!(file, "{}={}/{:.1}/{}", key, min_temp, avg_temp, max_temp)?;
                println!("{}={}/{:.1}/{}", key, min_temp, avg_temp, max_temp);
            }
            None => {
                println!("City: {}, No data", key);
            }
        }
    }

    Ok(())
}

pub fn solve_optimized(input_file: &str) -> std::io::Result<()> {
    let start = Instant::now();
    let file = File::open(input_file)?;
    let file_hash = input_file.split("_").last().unwrap().split(".").next().unwrap();
    let row_count = input_file.split("_").nth(1).unwrap().parse::<usize>().unwrap();
    println!("Solving test case file: {}", input_file);
    println!("File hash: {}", file_hash);
    println!("Row count: {}", row_count);

    let output_file = format!("testcases/answer_{}_{}.txt", row_count, file_hash);

    // Shared results map for all threads
    let results = Arc::new(Mutex::new(HashMap::<String, (f32, f64, f32, usize)>::new()));

    // Read the entire file and count the lines
    let mut reader = BufReader::new(file);
    let mut lines = Vec::new();
    let mut line = String::new();

    while reader.read_line(&mut line)? > 0 {
        lines.push(line.clone());
        line.clear();
    }

    println!("Read {} lines from file", lines.len());

    // Calculate chunks per worker
    let lines_per_worker = lines.len() / NUM_WORKERS;
    let mut handles = Vec::new();

    // Spawn worker threads
    for i in 0..NUM_WORKERS {
        let start_idx = i * lines_per_worker;
        let end_idx = if i == NUM_WORKERS - 1 {
            lines.len()
        } else {
            (i + 1) * lines_per_worker
        };

        let worker_lines = lines[start_idx..end_idx].to_vec();
        let worker_results = Arc::clone(&results);

        handles.push(thread::spawn(move || {
            let mut local_map = HashMap::<String, (f32, f64, f32, usize)>::new();

            for line in worker_lines {
                if let Some((city, temp_str)) = line.trim().split_once(";") {
                    if let Ok(temp) = temp_str.parse::<f32>() {
                        let city_string = city.to_string();
                        match local_map.get(&city_string) {
                            Some((min_temp, total_temp, max_temp, count)) => {
                                let new_min = if temp < *min_temp { temp } else { *min_temp };
                                let new_max = if temp > *max_temp { temp } else { *max_temp };
                                let new_total = total_temp + temp as f64;
                                let new_count = count + 1;
                                local_map.insert(city_string, (new_min, new_total, new_max, new_count));
                            }
                            None => {
                                local_map.insert(city_string, (temp, temp as f64, temp, 1));
                            }
                        }
                    }
                }
            }

            // Merge local results to global results
            let mut global_map = worker_results.lock().unwrap();
            for (city, (min_temp, total_temp, max_temp, count)) in local_map {
                match global_map.get(&city) {
                    Some((g_min, g_total, g_max, g_count)) => {
                        let new_min = if min_temp < *g_min { min_temp } else { *g_min };
                        let new_max = if max_temp > *g_max { max_temp } else { *g_max };
                        let new_total = g_total + total_temp;
                        let new_count = g_count + count;
                        global_map.insert(city, (new_min, new_total, new_max, new_count));
                    }
                    None => {
                        global_map.insert(city, (min_temp, total_temp, max_temp, count));
                    }
                }
            }

            println!("Worker {} completed", i);
        }));
    }

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }

    // Write results to file
    let global_results = results.lock().unwrap();
    let mut output = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(output_file)?;

    let mut keys: Vec<String> = global_results.keys().cloned().collect();
    keys.sort();

    for key in keys {
        if let Some((min_temp, total_temp, max_temp, count)) = global_results.get(&key) {
            let mut avg_temp = (((total_temp / *count as f64) * 10.0).round()) / 10.0;
            if avg_temp == 0.0 {
                avg_temp = 0.0;
            }
            writeln!(output, "{}={}/{:.1}/{}", key, min_temp, avg_temp, max_temp)?;
        }
    }

    let duration = start.elapsed();
    println!("Processing completed in {:.2?}", duration);

    Ok(())
}

