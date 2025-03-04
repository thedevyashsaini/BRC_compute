mod testcase;
mod benchmark;

use std::process::Command;
use std::io::BufRead;

const TESTCASE_PATH: &str = "testcases";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let level: f32 = std::env::var("LEVEL")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .unwrap_or(1000.0);

    println!("Tryna generate test case for level: {}", level);

    let num_rows: usize = (level * 1_000_000.0) as usize;
    let testcase_path: &std::path::Path = std::path::Path::new(TESTCASE_PATH);
    let testcase_pattern: String = format!(
        "./{}/testcase_{}_*.txt",
        testcase_path.to_str().unwrap(),
        num_rows
    );
    let mut testcase_id: String = String::new();

    if let Ok(file) = glob::glob(&testcase_pattern) {
        let files: Vec<_> = file.collect::<Result<Vec<_>, _>>().unwrap();
        if !files.is_empty() {
            let testcase_file: &str = files[0].to_str().unwrap();
            println!(
                "Test case file already exists: {}. Delete that shit if u want.",
                testcase_file
            );
            testcase_id = testcase_file
                .split("_")
                .last()
                .unwrap()
                .split(".")
                .next()
                .unwrap()
                .to_string();
            testcase::solver::solve_optimized(testcase_file).unwrap();
        } else {
            let testcase_file: String = testcase::generator::generate_testcase(num_rows).await?;
            let testcase_file_path: std::path::PathBuf = testcase_path.join(&testcase_file);
            testcase_id = testcase_file
                .split("_")
                .last()
                .unwrap()
                .split(".")
                .next()
                .unwrap()
                .to_string();
            println!(
                "Generated test case file: {}",
                testcase_file_path.to_str().unwrap()
            );
            testcase::solver::solve_optimized(testcase_file_path.to_str().unwrap()).unwrap();
        }
    }

    let source_path: String = format!(
        "./{}/testcase_{}_{}.txt",
        testcase_path.to_str().unwrap(),
        num_rows,
        testcase_id
    );

    let destination_dir: &str = "./src";

    std::fs::create_dir_all(destination_dir)?;
    if !std::path::Path::new(destination_dir).exists() {
        println!("Failed to create directory: {}", destination_dir);
        return Ok(());
    }

    let destination_path: String = format!("{}/testcase.txt", destination_dir);
    std::fs::remove_file(&destination_path).unwrap_or_default();

    if !std::path::Path::new(&source_path).exists() {
        println!("Source file does not exist: {}", source_path);
        return Ok(());
    }

    match std::fs::copy(&source_path, &destination_path) {
        Ok(bytes_copied) => {
            println!(
                "Attempting to copy from {} to {}",
                source_path, destination_path
            );
            println!(
                "Copied testcase to source directory. Bytes copied: {}",
                bytes_copied
            );

            if let Ok(metadata) = std::fs::metadata(&destination_path) {
                println!("Destination file size: {} bytes", metadata.len());
            } else {
                println!("Failed to verify destination file");
            }
        }
        Err(e) => {
            println!("Failed to copy testcase file: {}", e);
            println!("Current working directory: {:?}", std::env::current_dir()?);
        }
    }

    std::env::set_current_dir("src")?;

    println!("Running unbenchmarked test...");

    let mut child: std::process::Child = Command::new("python3.13")
        .args(["-X", "gil=0", "main.py"])
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .expect("Failed to run main.py! Surely this mf fked up something.");

    let status: std::process::ExitStatus = child
        .wait()
        .expect("Failed to wait for Python process to complete");

    println!("Process exited: {}", status);

    println!("Testing output...");

    let test_output_file = std::fs::File::open("./output.txt")
        .expect("Failed to open test output file");
    let test_output_reader = std::io::BufReader::new(test_output_file);
    let mut test_results: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();
    
    for line in std::io::BufReader::new(test_output_reader).lines() {
        let line = line.expect("Failed to read line");
        if line.trim().is_empty() { continue; }
        
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() != 2 {
            panic!("Malformed line in test output: {}", line);
        }
        
        let city = parts[0].to_string();
        let values: Vec<f64> = parts[1]
            .split('/')
            .map(|v| v.parse::<f64>()
                .unwrap_or_else(|_| panic!("Failed to parse value in test output: {}", v)))
            .collect();
            
        test_results.insert(city, values);
    }

    let expected_output_file = std::fs::File::open(
        format!("../{}/answer_{}_{}.txt", 
            testcase_path.to_str().unwrap(), 
            num_rows, 
            testcase_id
        )
    ).expect("Failed to open expected output file");
    
    let expected_output_reader = std::io::BufReader::new(expected_output_file);
    let mut expected_results: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();
    
    for line in std::io::BufReader::new(expected_output_reader).lines() {
        let line = line.expect("Failed to read line");
        if line.trim().is_empty() { continue; }
        
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() != 2 {
            panic!("Malformed line in expected output: {}", line);
        }
        
        let city = parts[0].to_string();
        let values: Vec<f64> = parts[1]
            .split('/')
            .map(|v| v.parse::<f64>()
                .unwrap_or_else(|_| panic!("Failed to parse value in expected output: {}", v)))
            .collect();
            
        expected_results.insert(city, values);
    }

    if test_results.len() != expected_results.len() {
        panic!("Number of cities mismatch: expected {} cities, got {}", 
            expected_results.len(), test_results.len());
    }

    for (city, expected_values) in &expected_results {
        match test_results.get(city) {
            Some(test_values) => {
                if test_values.len() != expected_values.len() {
                    panic!("Number of values mismatch for city {}: expected {}, got {}", 
                        city, expected_values.len(), test_values.len());
                }
                
                for (i, (test_val, expected_val)) in test_values.iter().zip(expected_values.iter()).enumerate() {
                    const EPSILON: f64 = 1e-6;
                    if (test_val - expected_val).abs() > EPSILON {
                        panic!("Value mismatch for city {} at position {}: expected {}, got {}", 
                            city, i, expected_val, test_val);
                    }
                }
            }
            None => {
                panic!("Missing city {} in test output", city);
            }
        }
    }

    for city in test_results.keys() {
        if !expected_results.contains_key(city) {
            panic!("Unexpected city {} in test output", city);
        }
    }

    println!("All tests passed successfully!");

    println!("Running benchmark...");

    let mut child: std::process::Child = Command::new("python3.13")
        .args(["-X","gil=0","-m","pyperf","command","-o","../bench.json","-p","1","--","python","src/main.py"])
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .expect("Failed to run main.py with benchmark! Surely this mf fked up something.");

    let status: std::process::ExitStatus = child
        .wait()
        .expect("Failed to wait for Python process to complete");

    println!("Process exited: {}", status);

    println!("Benchmarking output...");

    let benchmark_output_file: std::fs::File = match std::fs::File::open("../bench.json") {
        Ok(file) => { file },
        Err(err) => {
            panic!("Failed to open benchmark output file: {}", err);
        },
    };

    let benchmark_output_reader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(benchmark_output_file);
    let benchmark_output: serde_json::Value = serde_json::from_reader(benchmark_output_reader)?;

    println!("Benchmark output: {}", benchmark_output);

    let parsed_benchmark: (benchmark::parser::BenchmarkStats, serde_json::Value) = benchmark::parser::parse(&benchmark_output.to_string())?;
    
    println!("Parsed benchmark: {:?}", parsed_benchmark);

    let benchmark_output_file: std::fs::File = match std::fs::File::open("../bench_parsed.json") {
        Ok(file) => { file },
        Err(err) => {
            panic!("Failed to open parsed benchmark output file: {}", err);
        },
    };

    let benchmark_output_writer: std::io::BufWriter<std::fs::File> = std::io::BufWriter::new(benchmark_output_file);
    serde_json::to_writer(benchmark_output_writer, &parsed_benchmark)?;

    println!("Parsed benchmark written to file!");

    Ok(())
}
