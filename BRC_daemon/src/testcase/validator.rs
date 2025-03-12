use std::collections::HashMap;
use std::io::{self, BufRead};
use std::fs::File;

pub struct ValidationResult {
    pub success: bool,
    pub message: String,
}

pub fn validate_output(expected_output_lines: &[String], test_output_path: &str) -> io::Result<ValidationResult> {
    println!("Testing output...");
    
    let test_output_file = match File::open(test_output_path) {
        Ok(f) => f,
        Err(e) => {
            return Ok(ValidationResult {
                success: false,
                message: format!("Failed to open test output file: {}", e),
            });
        }
    };
    
    let test_output_reader = io::BufReader::new(test_output_file);
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
        return Ok(ValidationResult {
            success: false,
            message: format!(
                "Number of cities mismatch: expected {} cities, got {}",
                expected_output_lines.len(),
                test_output_lines.len()
            ),
        });
    }
    
    let mut test_city_positions: HashMap<String, usize> = HashMap::new();
    for (i, line) in test_output_lines.iter().enumerate() {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() != 2 {
            return Ok(ValidationResult {
                success: false,
                message: format!("Malformed line in test output: {}", line),
            });
        }
        test_city_positions.insert(parts[0].to_string(), i);
    }
    
    for (expected_pos, expected_line) in expected_output_lines.iter().enumerate() {
        let expected_parts: Vec<&str> = expected_line.split('=').collect();
        if expected_parts.len() != 2 {
            return Ok(ValidationResult {
                success: false,
                message: format!("Malformed line in expected output: {}", expected_line),
            });
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
                    return Ok(ValidationResult {
                        success: false,
                        message: format!(
                            "City '{}' is out of order: expected at position {}, found at position {}",
                            expected_city, expected_pos, actual_pos
                        ),
                    });
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
                    return Ok(ValidationResult {
                        success: false,
                        message: format!(
                            "Number of values mismatch for city {}: expected {}, got {}",
                            expected_city,
                            expected_values.len(),
                            test_values.len()
                        ),
                    });
                }
    
                // Check each value
                for (i, (test_val, expected_val)) in test_values.iter().zip(expected_values.iter()).enumerate() {
                    const EPSILON: f64 = 1e-6;
                    if (test_val - expected_val).abs() > EPSILON {
                        return Ok(ValidationResult {
                            success: false,
                            message: format!(
                                "Value mismatch for city {} at position {}: expected {}, got {}",
                                expected_city, i, expected_val, test_val
                            ),
                        });
                    }
                }
            }
            None => {
                return Ok(ValidationResult {
                    success: false,
                    message: format!("Missing city {} in test output", expected_city),
                });
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
            return Ok(ValidationResult {
                success: false,
                message: format!("Unexpected city {} in test output", test_city),
            });
        }
    }
    
    Ok(ValidationResult {
        success: true,
        message: "All tests passed successfully! Output matches expected format and order.".to_string(),
    })
}