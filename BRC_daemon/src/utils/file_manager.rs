use std::path::Path;
use std::io::{self, BufRead};
use std::fs;
use crate::testcase::solver;
use crate::testcase::generator;

pub const TESTCASE_PATH: &str = "testcases";

#[allow(dead_code)]
pub struct TestCaseInfo {
    pub source_path: String,
    pub destination_path: String,
    pub testcase_id: String,
    pub num_rows: usize,
}

pub fn copy_testcase_to_src_dir(num_rows: usize, testcase_id: &str) -> io::Result<TestCaseInfo> {
    let testcase_path = Path::new(TESTCASE_PATH);
    let source_path = format!(
        "./{}/testcase_{}_{}.txt",
        testcase_path.to_str().unwrap(),
        num_rows,
        testcase_id
    );
    let destination_dir = "./src";
    let destination_path = format!("{}/testcase.txt", destination_dir);

    // Ensure destination directory exists
    fs::create_dir_all(destination_dir)?;
    if !Path::new(destination_dir).exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Failed to create directory: {}", destination_dir)
        ));
    }

    // Clean up old file if it exists
    fs::remove_file(&destination_path).unwrap_or_default();

    // Check if source file exists
    if !Path::new(&source_path).exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source file does not exist: {}", source_path)
        ));
    }

    // Copy the file
    match fs::copy(&source_path, &destination_path) {
        Ok(bytes_copied) => {
            println!(
                "Copied testcase to source directory. Bytes copied: {}",
                bytes_copied
            );

            if let Ok(metadata) = fs::metadata(&destination_path) {
                println!("Destination file size: {} bytes", metadata.len());
            }
            
            Ok(TestCaseInfo {
                source_path,
                destination_path,
                testcase_id: testcase_id.to_string(),
                num_rows,
            })
        },
        Err(e) => {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to copy testcase file: {}", e)
            ))
        }
    }
}

pub async fn find_or_create_testcase(num_rows: usize) -> io::Result<String> {
    let testcase_path = Path::new(TESTCASE_PATH);
    let testcase_pattern = format!(
        "./{}/testcase_{}_*.txt",
        testcase_path.to_str().unwrap(),
        num_rows
    );
    
    // Check if testcase already exists
    if let Ok(file) = glob::glob(&testcase_pattern) {
        let files: Vec<_> = file.collect::<Result<Vec<_>, _>>()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, 
                format!("Failed to collect testcase files: {}", e)))?;
                
        if !files.is_empty() {
            let testcase_file = files[0].to_str().unwrap();
            println!(
                "Test case file already exists: {}. Using existing file.",
                testcase_file
            );
            
            let testcase_id = testcase_file
                .split("_")
                .last()
                .unwrap()
                .split(".")
                .next()
                .unwrap()
                .to_string();
                
            // Solve the testcase
            solver::solve_testcase(testcase_file)?;
            return Ok(testcase_id);
        }
    }
    
    // Generate new testcase if none exists
    let testcase_file = generator::generate_testcase(num_rows).await?;
    let testcase_file_path = testcase_path.join(&testcase_file);
    
    let testcase_id = testcase_file
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
    
    solver::solve_testcase(testcase_file_path.to_str().unwrap())?;
    
    Ok(testcase_id)
}

pub fn read_lines_from_file(file_path: &str) -> io::Result<Vec<String>> {
    let file = fs::File::open(file_path)?;
    let reader = io::BufReader::new(file);
    reader.lines()
        .map(|line| line.map_err(|e| io::Error::new(io::ErrorKind::Other, e)))
        .collect()
}

pub fn ensure_output_dir() -> io::Result<()> {
    fs::create_dir_all("./output")
}