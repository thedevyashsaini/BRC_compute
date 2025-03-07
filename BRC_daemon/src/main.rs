mod benchmark;
mod testcase;

use std::io::BufRead;
use std::process::Command;

const TESTCASE_PATH: &str = "testcases";

async fn write_status(success: bool, message: &str) -> std::io::Result<()> {
    let status_file_path = if std::path::Path::new("src").exists() && !std::env::current_dir().unwrap().ends_with("src") {
        "./output/status.json"
    } else {
        "../output/status.json"
    };
    let status_file: std::fs::File = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(status_file_path)
        .expect("Failed to open status file for writing");

    let status_writer: std::io::BufWriter<std::fs::File> = std::io::BufWriter::new(status_file);
    let json_status: serde_json::Value = serde_json::json!({
        "success": success,
        "message": message
    });
    serde_json::to_writer(status_writer, &json_status)?;
    Ok(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let level: f32 = match std::env::var("LEVEL")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
    {
        Ok(val) => val,
        Err(e) => {
            write_status(false, &format!("Failed to parse LEVEL env var: {}", e)).await?;
            return Ok(());
        }
    };

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
        let files: Vec<_> = match file.collect::<Result<Vec<_>, _>>() {
            Ok(f) => f,
            Err(e) => {
                write_status(false, &format!("Failed to collect testcase files: {}", e)).await?;
                return Ok(());
            }
        };
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
            testcase::solver::solve_testcase(testcase_file)?;
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
            testcase::solver::solve_testcase(testcase_file_path.to_str().unwrap())?;
        }
    }

    if let Err(e) = std::fs::create_dir_all("./output") {
        eprintln!("Failed to create output directory: {}", e);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create output directory: {}", e),
        ));
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
        write_status(
            false,
            &format!("Failed to create directory: {}", destination_dir),
        )
        .await?;
        println!("Failed to create directory: {}", destination_dir);
        return Ok(());
    }

    let destination_path: String = format!("{}/testcase.txt", destination_dir);
    std::fs::remove_file(&destination_path).unwrap_or_default();

    if !std::path::Path::new(&source_path).exists() {
        write_status(
            false,
            &format!("Source file does not exist: {}", source_path),
        )
        .await
        .unwrap();
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
            write_status(false, &format!("Failed to copy testcase file: {}", e)).await?;
            println!("Failed to copy testcase file: {}", e);
            panic!("Current working directory: {:?}", std::env::current_dir()?);
        }
    }

    let expected_output_file_path = format!(
        "{}/answer_{}_{}.txt",
        testcase_path.to_str().unwrap(),
        num_rows,
        testcase_id
    );

    let expected_output_file = match std::fs::File::open(&expected_output_file_path) {
        Ok(f) => f,
        Err(e) => {
            write_status(
                false,
                &format!("Failed to open expected output file: {}", e),
            )
            .await?;
            panic!("Failed to open expected output file: {}", e);
        }
    };

    let expected_output_reader = std::io::BufReader::new(expected_output_file);
    let expected_output_lines: Vec<String> = expected_output_reader
        .lines()
        .map(|l| l.expect("Failed to read line"))
        .collect();

    match std::fs::remove_file(expected_output_file_path) {
        Err(e) => {
            write_status(false, &format!("Failed to delete output file: {}", e)).await?;
            return Ok(());
        }
        _ => {}
    };


    if let Err(e) = std::env::set_current_dir("src") {
        write_status(false, &format!("Failed to change directory to src: {}", e)).await?;
        return Ok(());
    }

    println!("Running unbenchmarked test...");

    let mut child: std::process::Child = Command::new("python3.13")
        .args(["-X", "gil=0", "main.py"])
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .expect("Failed to run main.py! Surely this mf fked up something.");

    let status = match child.wait() {
        Ok(status) => status,
        Err(e) => {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    write_status(false, &format!("Failed to wait for Python process: {}", e))
                        .await
                        .unwrap();
                    panic!("Failed to wait for Python process: {}", e);
                })
            });
            std::process::exit(1);
        }
    };

    if !status.success() {
        write_status(
            false,
            &format!("Python process exited with non-zero status: {}", status),
        )
        .await?;
        panic!("Python process exited with non-zero status: {}", status);
    }

    println!("Process exited: {}", status);

    println!("Testing output...");
    
    let test_output_file = match std::fs::File::open("./output.txt") {
        Ok(f) => f,
        Err(e) => {
            write_status(false, &format!("Failed to open test output file: {}", e)).await?;
            return Ok(());
        }
    };
    
    let test_output_reader = std::io::BufReader::new(test_output_file);
    let test_output_lines: Vec<String> = test_output_reader
        .lines()
        .filter_map(|line| {
            let line = line.expect("Failed to read line");
            if line.trim().is_empty() {
                None
            } else {
                Some(line)
            }
        })
        .collect();
    
    if test_output_lines.len() != expected_output_lines.len() {
        write_status(
            false,
            &format!(
                "Number of cities mismatch: expected {} cities, got {}",
                expected_output_lines.len(),
                test_output_lines.len()
            ),
        )
        .await?;
        panic!(
            "Number of cities mismatch: expected {} cities, got {}",
            expected_output_lines.len(),
            test_output_lines.len()
        );
    }
    
    let mut test_city_positions: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (i, line) in test_output_lines.iter().enumerate() {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() != 2 {
            write_status(false, &format!("Malformed line in test output: {}", line)).await?;
            panic!("Malformed line in test output: {}", line);
        }
        test_city_positions.insert(parts[0].to_string(), i);
    }
    
    for (expected_pos, expected_line) in expected_output_lines.iter().enumerate() {
        let expected_parts: Vec<&str> = expected_line.split('=').collect();
        if expected_parts.len() != 2 {
            write_status(
                false,
                &format!("Malformed line in expected output: {}", expected_line),
            )
            .await?;
            panic!("Malformed line in expected output: {}", expected_line);
        }
    
        let expected_city = expected_parts[0];
        let expected_values: Vec<f64> = expected_parts[1]
            .split('/')
            .map(|v| {
                v.parse::<f64>().unwrap_or_else(|_| {
                    panic!("Failed to parse value in expected output: {}", v);
                })
            })
            .collect();
    
        // Check if city exists and get its position
        match test_city_positions.get(expected_city) {
            Some(&actual_pos) => {
                // City exists, but check if it's in the correct position
                if actual_pos != expected_pos {
                    write_status(
                        false,
                        &format!(
                            "City '{}' is out of order: expected at position {}, found at position {}",
                            expected_city, expected_pos, actual_pos
                        ),
                    )
                    .await?;
                    panic!(
                        "City '{}' is out of order: expected at position {}, found at position {}",
                        expected_city, expected_pos, actual_pos
                    );
                }
    
                // Now check the values
                let test_parts: Vec<&str> = test_output_lines[actual_pos].split('=').collect();
                let test_values: Vec<f64> = test_parts[1]
                    .split('/')
                    .map(|v| {
                        v.parse::<f64>().unwrap_or_else(|_| {
                            panic!("Failed to parse value in test output: {}", v);
                        })
                    })
                    .collect();
    
                if test_values.len() != expected_values.len() {
                    write_status(
                        false,
                        &format!(
                            "Number of values mismatch for city {}: expected {}, got {}",
                            expected_city,
                            expected_values.len(),
                            test_values.len()
                        ),
                    )
                    .await?;
                    panic!(
                        "Number of values mismatch for city {}: expected {}, got {}",
                        expected_city,
                        expected_values.len(),
                        test_values.len()
                    );
                }
    
                // Check each value
                for (i, (test_val, expected_val)) in test_values.iter().zip(expected_values.iter()).enumerate() {
                    const EPSILON: f64 = 1e-6;
                    if (test_val - expected_val).abs() > EPSILON {
                        write_status(
                            false,
                            &format!(
                                "Value mismatch for city {} at position {}: expected {}, got {}",
                                expected_city, i, expected_val, test_val
                            ),
                        )
                        .await?;
                        panic!(
                            "Value mismatch for city {} at position {}: expected {}, got {}",
                            expected_city, i, expected_val, test_val
                        );
                    }
                }
            }
            None => {
                write_status(
                    false,
                    &format!("Missing city {} in test output", expected_city),
                )
                .await?;
                panic!("Missing city {} in test output", expected_city);
            }
        }
    }
    
    // Check for any unexpected cities
    for test_line in test_output_lines.iter() {
        let test_city = test_line.split('=').next().unwrap();
        if !expected_output_lines
            .iter()
            .any(|line| line.starts_with(test_city))
        {
            write_status(
                false,
                &format!("Unexpected city {} in test output", test_city),
            )
            .await?;
            panic!("Unexpected city {} in test output", test_city);
        }
    }
    
    println!("All tests passed successfully! Output matches expected format and order.");
    
    println!("Running benchmark...");

    let benchmark_file_name: &str = "../output/bench.json";

    std::fs::remove_file(benchmark_file_name).unwrap_or_default();

    let mut child: std::process::Child = Command::new("python3.13")
        .args([
            "-X",
            "gil=0",
            "-m",
            "pyperf",
            "command",
            "-o",
            benchmark_file_name,
            "-p",
            "1",
            "--",
            "python3.13",
            "-X",
            "gil=0",
            "main.py",
        ])
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .expect("Failed to run main.py with benchmark! Surely this mf fked up something.");

    let status: std::process::ExitStatus = child
        .wait()
        .expect("Failed to wait for Python process to complete");

    println!("Process exited: {}", status);

    println!("Fetching benchmark output...");

    let benchmark_output_file: std::fs::File = match std::fs::File::open(benchmark_file_name) {
        Ok(file) => file,
        Err(err) => {
            panic!("Failed to open benchmark output file: {}", err);
        }
    };

    let benchmark_output_reader: std::io::BufReader<std::fs::File> =
        std::io::BufReader::new(benchmark_output_file);
    let benchmark_output: serde_json::Value = match serde_json::from_reader(benchmark_output_reader)
    {
        Ok(b) => b,
        Err(e) => {
            write_status(false, &format!("Failed to parse benchmark output: {}", e)).await?;
            return Ok(());
        }
    };

    println!("Got benchmark output!\nParsing benchmark...");

    let parsed_benchmark: (benchmark::parser::BenchmarkStats, serde_json::Value) =
        match benchmark::parser::parse(benchmark_output) {
            Ok(p) => p,
            Err(e) => {
                write_status(false, &format!("Failed to parse benchmark: {}", e)).await?;
                return Ok(());
            }
        };

    println!("Parsed benchmark!");

    print!(
        "Average runtime: {:.6} ms\n",
        parsed_benchmark.0.get_mean() / 1000.0
    );

    // create file for parsed benchmark output
    let bench_parsed_path: &str = "../output/bench_parsed.json";
    if !std::path::Path::new(bench_parsed_path).exists() {
        std::fs::File::create(bench_parsed_path)?;
    }

    let benchmark_output_file: std::fs::File = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(bench_parsed_path)
        .unwrap();

    let benchmark_output_writer: std::io::BufWriter<std::fs::File> =
        std::io::BufWriter::new(benchmark_output_file);
    if let Err(e) = serde_json::to_writer_pretty(benchmark_output_writer, &parsed_benchmark) {
        write_status(false, &format!("Failed to write benchmark results: {}", e))
            .await
            .unwrap();
        return Ok(());
    }

    println!("Parsed benchmark written to file!");

    write_status(true, "It fking worked!!").await.unwrap();

    Ok(())
}
