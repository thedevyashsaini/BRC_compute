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
