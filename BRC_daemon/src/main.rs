mod testcase;

use std::process::Command;

const TESTCASE_PATH: &str = "testcases";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Read from environment variable with a default value of 1000
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

    // TODO: Execute main.py file with pyperf
    // check if the output matches the expected output
    // if it does, then print the time taken to solve the test case, write success to result.json file
    // if it doesn't, show testcase failed and write error to a result.json file

    let mut child: std::process::Child = Command::new("python3.13")
        .args(["-X", "gil=0", "src/main.py"])
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .expect("Failed to run main.py! Surely this mf fked up something.");

    let status: std::process::ExitStatus = child
        .wait()
        .expect("Failed to wait for Python process to complete");

    println!("Process exited: {}", status);

    // let _output = Command::new("python3.13")
    //     .args(["-X","gil=0","-m","pyperf","command","-o","./bench.json","-p","1","--","python","src/main.py"])
    //     .output()
    //     .expect("Failed to run benchmark command! Surely this mf fked up something.");

    Ok(())
}
