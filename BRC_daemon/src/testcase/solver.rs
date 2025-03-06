use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

const NUM_WORKERS: usize = 10;

#[allow(dead_code)]
pub fn solve_testcase(input_file: &str) -> std::io::Result<()> {
    let file = File::open(input_file)?;
    let file_hash = input_file.split("_").last().unwrap().split(".").next().unwrap();
    let row_count = input_file.split("_").nth(1).unwrap().parse::<usize>().unwrap();
    println!("Solving test case file: {}", input_file);
    println!("File hash: {}", file_hash);
    println!("Row count: {}", row_count);

    let reader: BufReader<File> = BufReader::new(file);

    let output_file: String = format!("testcases/answer_{}_{}.txt", row_count, file_hash);
    let mut file: File = OpenOptions::new().create(true).write(true).truncate(true).open(output_file)?;

    // Use integers to store temperatures (multiplied by 10)
    let mut records: HashMap<String, (i64, i64, i64, usize)> = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        let (city, temp_str) = line.split_once(";").unwrap();

        // Parse as float first, then convert to integer by multiplying by 10
        let temp_float: f64 = temp_str.parse().unwrap();
        let temp = (temp_float * 10.0).round() as i64;
        let city_string = city.to_string();

        match records.get(&city_string) {
            Some(temps) => {
                let min_temp = if temp < temps.0 { temp } else { temps.0 };
                let total_temp = temps.1 + temp;
                let max_temp = if temp > temps.2 { temp } else { temps.2 };
                let count = temps.3 + 1;
                records.insert(city_string, (min_temp, total_temp, max_temp, count));
            }
            None => {
                records.insert(city_string, (temp, temp, temp, 1));
            }
        }
    }

    let mut keys: Vec<String> = records.keys().cloned().collect();
    keys.sort();

    for key in keys {
        let value = records.get(&key);

        match value {
            Some((min_temp, total_temp, max_temp, count)) => {
                let avg = (*total_temp as f64 / *count as f64).ceil();
                println!("Total Temp for {}: {} with avg of {}", key, total_temp, avg);
                // Format with one decimal place by dividing by 10
                writeln!(
                    file,
                    "{}={:.1}/{:.1}/{:.1}",
                    key,
                    *min_temp as f64 / 10.0,
                    avg / 10.0,
                    *max_temp as f64 / 10.0
                )?;
                // println!(
                //     "{}={:.1}/{:.1}/{:.1}",
                //     key,
                //     *min_temp as f64 / 10.0,
                //     avg as f64 / 10.0,
                //     *max_temp as f64 / 10.0
                // );
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
    
    let output_file = format!("testcases/answer_{}_{}.txt", row_count, file_hash);
    let results = Arc::new(Mutex::new(HashMap::<String, (f32, f64, f32, usize)>::new()));
    let mut reader = BufReader::with_capacity(8 * 1024 * 1024, file); 
    
    let chunk_size = std::cmp::min(10_000_000, row_count / 100); 
    let mut lines = Vec::with_capacity(chunk_size);
    let mut current_line = String::new();
    let mut handles = Vec::new();
    let mut total_processed = 0;

    loop {
        lines.clear();
        for _ in 0..chunk_size {
            current_line.clear();
            match reader.read_line(&mut current_line) {
                Ok(0) => break, // EOF
                Ok(_) => lines.push(current_line.clone()),
                Err(e) => return Err(e),
            }
        }

        if lines.is_empty() {
            break;
        }

        total_processed += lines.len();
        println!("Processing chunk of {} lines. Total processed: {}", lines.len(), total_processed);

        // Process chunk with workers
        let lines_per_worker = lines.len() / NUM_WORKERS;
        
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
                process_chunk(worker_lines, worker_results);
            }));
        }

        // Wait for current chunk to complete
        for handle in handles.drain(..) {
            handle.join().unwrap();
        }
    }

    // Write results
    write_results(&results, &output_file)?;

    let duration = start.elapsed();
    println!("Processing completed in {:.2?}", duration);
    Ok(())
}

fn process_chunk(lines: Vec<String>, results: Arc<Mutex<HashMap<String, (f32, f64, f32, usize)>>>) {
    let mut local_map = HashMap::new();

    for line in lines {
        if let Some((city, temp_str)) = line.trim().split_once(";") {
            if let Ok(temp) = temp_str.parse::<f32>() {
                let entry = local_map
                    .entry(city.to_string())
                    .or_insert((temp, temp as f64, temp, 0));
                entry.0 = entry.0.min(temp);
                entry.1 += temp as f64;
                entry.2 = entry.2.max(temp);
                entry.3 += 1;
            }
        }
    }

    // Merge results periodically
    let mut global_map = results.lock().unwrap();
    for (city, (min_temp, total_temp, max_temp, count)) in local_map {
        let entry = global_map
            .entry(city)
            .or_insert((min_temp, total_temp, max_temp, count));
        entry.0 = entry.0.min(min_temp);
        entry.1 += total_temp;
        entry.2 = entry.2.max(max_temp);
        entry.3 += count;
    }
}

fn write_results(results: &Arc<Mutex<HashMap<String, (f32, f64, f32, usize)>>>, output_file: &str) -> std::io::Result<()> {
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
    Ok(())
}