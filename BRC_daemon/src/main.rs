mod testcase;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let testcase_file = testcase::generator::generate_testcase(1_000_000_000).await?;
    println!("Generated test case file: {}", testcase_file);
    Ok(())
}
