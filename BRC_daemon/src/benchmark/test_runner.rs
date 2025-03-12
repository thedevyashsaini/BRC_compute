use std::process::Stdio;
use tokio::process::Command as TokioCommand;
use tokio::time::{timeout, Duration, Instant};
use std::io;

pub struct TestResult {
    pub success: bool,
    pub message: String,
    pub runtime: Option<u64>
}

pub async fn run_python_test(timeout_seconds: u64) -> io::Result<TestResult> {
    println!("Running unbenchmarked test...");

    // Start the Python process
    let mut child = TokioCommand::new("python")
        .args(["-X", "gil=0", "main.py"])
        .current_dir("src")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, 
            format!("Failed to run main.py: {}", e)))?;

    let start_time = Instant::now();

    // Timeout wrapper
    match timeout(Duration::from_secs(timeout_seconds), async {
        let status = child.wait().await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, 
                format!("Failed to wait for process: {}", e)))?;
        println!("Process exited with: {}", status);
        Ok::<_, io::Error>(status)
    }).await {
        Ok(status_result) => {
            match status_result {
                Ok(status) => {
                    let elapsed_ms: u64 = start_time.elapsed().as_secs();
                    // Process finished before timeout
                    if !status.success() {
                        return Ok(TestResult {
                            success: false,
                            message: format!("Python process failed with non-zero exit code: {}", status),
                            runtime: Some(elapsed_ms)
                        });
                    }
                    println!("Process completed successfully in {}s", elapsed_ms);
                    Ok(TestResult {
                        success: true,
                        message: "Test completed successfully".to_string(),
                        runtime: Some(elapsed_ms)
                    })
                },
                Err(e) => Ok(TestResult {
                    success: false,
                    message: e.to_string(),
                    runtime: None
                }),
            }
        },
        Err(_) => {
            // Timeout triggered
            println!("Timer won the race! Process took too long, killing it.");

            if let Err(e) = child.kill().await {
                eprintln!("Failed to kill process: {}", e);
            } else {
                let _ = child.wait().await;
            }
            
            Ok(TestResult {
                success: false,
                message: format!("Process timed out after {} seconds", timeout_seconds),
                runtime: None
            })
        }
    }
}

pub fn run_benchmark(benchmark_file_name: &str, skip_calibration: bool) -> io::Result<std::process::ExitStatus> {
    println!("Running benchmark...");

    // Clean up old benchmark file if it exists
    std::fs::remove_file(benchmark_file_name).unwrap_or_default();

    let output_path: String = format!("../{}", benchmark_file_name);
    let mut args: Vec<&str> = vec![
        "-X", "gil=0",
        "-m", "pyperf",
        "command",
        "-o", &output_path,
        "-p", "1",
    ];

    println!("Skipping calibration in benchmark: {}", skip_calibration);

    if skip_calibration {
        args.push("--loops");
        args.push("1");
    }
    
    args.extend(["--", "python", "-X", "gil=0", "main.py"]);

    let mut child = std::process::Command::new("python")
        .args(args)
        .current_dir("src") 
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, 
            format!("Failed to run benchmark: {}", e)))?;

    let status = child.wait()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, 
            format!("Failed to wait for benchmark process: {}", e)))?;

    println!("Benchmark process exited: {}", status);
    Ok(status)
}