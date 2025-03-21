mod benchmark;
mod testcase;
mod utils;

use testcase::validator;
use benchmark::test_runner;
use utils::{file_manager, status};
use std::fs::OpenOptions;
use std::io;
use std::fs;

const TIMEOUT: u64 = 40;
const CALIBRATION_TIMEOUT: u64 = TIMEOUT + 1;

#[tokio::main]
async fn main() -> io::Result<()> {
    
    // Check if main.py exists
    let main_py_path = std::path::Path::new("src/main.py");
    if !main_py_path.exists() {
        status::write_status(false, "main.py does not exist").await?;
        return Err(io::Error::new(io::ErrorKind::NotFound, "main.py does not exist"));
    }

    // Parse level environment variable
    let level: f32 = match std::env::var("LEVEL")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
    {
        Ok(val) => val,
        Err(e) => {
            status::write_status(false, &format!("Failed to parse LEVEL env var: {}", e)).await?;
            return Ok(());
        }
    };

    println!("Generating test case for level: {}", level);

    // Calculate number of rows based on level
    let num_rows: usize = (level * 1_000_000.0) as usize;
    
    // Create output directory
    file_manager::ensure_output_dir()?;

    // Find or create a test case
    let testcase_id = match file_manager::find_or_create_testcase(num_rows).await {
        Ok(id) => id,
        Err(e) => {
            status::write_status(false, &format!("Failed to find or create testcase: {}", e)).await?;
            return Ok(());
        }
    };

    // Copy the test case to the src directory
    let _ = match file_manager::copy_testcase_to_src_dir(num_rows, &testcase_id) {
        Ok(info) => info,
        Err(e) => {
            status::write_status(false, &format!("Failed to copy testcase: {}", e)).await?;
            return Ok(());
        }
    };

    // Read expected output file
    let expected_output_file_path = format!(
        "{}/answer_{}_{}.txt",
        file_manager::TESTCASE_PATH,
        num_rows,
        testcase_id
    );

    let expected_output_lines = match file_manager::read_lines_from_file(&expected_output_file_path) {
        Ok(lines) => {
            fs::remove_file(expected_output_file_path)?;
            lines
        },
        Err(e) => {
            status::write_status(false, &format!("Failed to read expected output: {}", e)).await?;
            return Ok(());
        }
    };

    // Run the Python solution
    let test_result = test_runner::run_python_test(TIMEOUT).await?;
    if !test_result.success {
        status::write_status(false, &test_result.message).await?;
        return Err(io::Error::new(io::ErrorKind::Other, test_result.message));
    }

    let skip_calibration = test_result.runtime.is_none() || test_result.runtime.unwrap() >= CALIBRATION_TIMEOUT; 

    // Validate the output
    let validation_result = validator::validate_output(&expected_output_lines, "src/output.txt")?;
    if !validation_result.success {
        status::write_status(false, &validation_result.message).await?;
        return Err(io::Error::new(io::ErrorKind::Other, validation_result.message));
    }

    if let Err(error) = fs::remove_file("src/output.txt") {
        status::write_status(false, &format!("Failed to remove output file: {}", error)).await?;
        return Ok(());
    }

    println!("{}", validation_result.message);

    // Run benchmark
    let benchmark_file_name: &str = "output/bench.json";
    match test_runner::run_benchmark(benchmark_file_name, skip_calibration) {
        Ok(_) => {},
        Err(e) => {
            status::write_status(false, &format!("Failed to run benchmark: {}", e)).await?;
            return Ok(());
        }
    }

    // Parse benchmark results
    println!("Fetching benchmark output...");
    let benchmark_output = match fs::File::open(benchmark_file_name) {
        Ok(file) => {
            let reader = io::BufReader::new(file);
            match serde_json::from_reader(reader) {
                Ok(b) => b,
                Err(e) => {
                    status::write_status(false, &format!("Failed to parse benchmark output: {}", e)).await?;
                    return Ok(());
                }
            }
        },
        Err(e) => {
            status::write_status(false, &format!("Failed to open benchmark file: {}", e)).await?;
            return Ok(());
        }
    };

    println!("Got benchmark output!\nParsing benchmark...");
    let parsed_benchmark = match benchmark::parser::parse(benchmark_output) {
        Ok(p) => p,
        Err(e) => {
            status::write_status(false, &format!("Failed to parse benchmark: {}", e)).await?;
            return Ok(());
        }
    };

    print!("Average runtime: {:.6} ms\n", parsed_benchmark.0.get_mean() / 1000.0);

    // Write parsed benchmark to file
    let bench_parsed_path: &str = "output/bench_parsed.json";
    let benchmark_output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(bench_parsed_path)?;

    let benchmark_output_writer = io::BufWriter::new(benchmark_output_file);
    if let Err(e) = serde_json::to_writer_pretty(benchmark_output_writer, &parsed_benchmark) {
        status::write_status(false, &format!("Failed to write benchmark results: {}", e)).await?;
        return Ok(());
    }

    println!("Parsed benchmark written to file!");
    status::write_status(true, "Testing and benchmarking completed successfully").await?;

    Ok(())
}