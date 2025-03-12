use std::fs::OpenOptions;
use std::io;
use std::path::Path;
use serde_json;

pub async fn write_status(success: bool, message: &str) -> io::Result<()> {
    let status_file_path = if Path::new("src").exists() && !std::env::current_dir().unwrap().ends_with("src") {
        "./output/status.json"
    } else {
        "../output/status.json"
    };
    
    if let Some(parent_dir) = Path::new(status_file_path).parent() {
        if !parent_dir.exists() {
            std::fs::create_dir_all(parent_dir)?;
        }
    }
    
    let status_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(status_file_path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, 
            format!("Failed to open status file: {}", e)))?;

    let status_writer = io::BufWriter::new(status_file);
    let json_status = serde_json::json!({
        "success": success,
        "message": message
    });
    
    serde_json::to_writer(status_writer, &json_status)?;
    Ok(())
}