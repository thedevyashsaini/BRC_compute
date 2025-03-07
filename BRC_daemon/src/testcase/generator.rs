use std::{
    fs::File,
    io::{BufWriter, Write},
    sync::Arc,
    time::Instant,
};
use rand::{thread_rng, Rng};
use tokio::sync::mpsc;
use uuid::Uuid;

const CITIES: [&str; 221] = [
    "Gali-Makhian-Wali", "Mumbai", "Delhi", "Bangalore", "Hyderabad", "Ahmedabad", "Chennai", "Kolkata", "Pune", "Jaipur", "Lucknow",
    "Kanpur", "Nagpur", "Indore", "Thane", "Bhopal", "Visakhapatnam", "Patna", "Vadodara", "Ghaziabad", "Ludhiana",
    "Agra", "Nashik", "Ranchi", "Faridabad", "Meerut", "Rajkot", "Kalyan-Dombivli", "Vasai-Virar", "Varanasi", "Srinagar",
    "Aurangabad", "Dhanbad", "Amritsar", "Kotha", "Navi-Mumbai", "Allahabad", "Howrah", "Gwalior", "Jabalpur", "Coimbatore", "Vijayawada",
    "Jodhpur", "Madurai", "Raipur", "Kota", "Chandigarh", "Guwahati", "Solapur", "Hubballi-Dharwad", "Mysore", "Tiruchirappalli",
    "Bareilly", "Aligarh", "Tiruppur", "Moradabad", "Bhubaneswar", "Salem", "Warangal", "Guntur", "Bhiwandi", "Saharanpur",
    "Gorakhpur", "Bikaner", "Amravati", "Lula-Ahir", "Jamshedpur", "Bhilai", "Cuttack", "Firozabad", "Kochi", "Nellore", "Bhavnagar",
    "Dehradun", "Durgapur", "Asansol", "Rourkela", "Tatti-Khana", "Nanded", "Kolhapur", "Ajmer", "Akola", "Gulbarga", "Ujjain", "Bhosari",
    "Jamnagar", "Loni", "Siliguri", "Jhansi", "Ulhasnagar", "Jammu", "Sangli-Miraj-&-Kupwad", "Belagavi", "Mangalore", "Erode",
    "Tirunelveli", "Malegaon", "Gaya", "Udaipur", "Maheshtala", "Davanagere", "Kozhikode", "Kurnool", "Bokaro", "Rajahmundry",
    "South Dumdum", "Gopalpur", "Hajipur", "Bilaspur", "Muzaffarnagar", "Mathura", "Patiala", "Sagar", "Vellore", "Bijapur",
    "Shimoga", "Burhanpur", "Panipat", "Darbhanga", "Dibrugarh", "Tumkur", "Bally", "Muzaffarpur", "Ambattur", "North-Dumdum", "Cumbum",
    "Rohtak", "Bhagalpur", "Kollam", "Dewas", "Nizamabad", "Shahjahanpur", "Bharatpur", "Bhusawal", "Ratlam", "Chhindwara",
    "Dindigul", "Rewa", "Hajipur", "Ambala", "Korba", "Purnia", "Satna", "Kakinada", "Bhimavaram", "Ongole", "Kundara",
    "Hosur", "Adoni", "Machilipatnam", "Proddatur", "Tiruvannamalai", "Sikar", "Gondia", "Bhiwani", "Sirsa", "Karaikal",
    "Chittoor", "Dibrugarh", "Tezpur", "Shillong", "Imphal", "Aizawl", "Itanagar", "Kohima", "Agartala", "Gangtok",
    "Kavaratti", "Port-Blair", "Daman", "Silvassa", "Panaji", "Margao", "Mapusa", "Porvorim", "Karwar", "Hospet", "Lulla-Nagar",
    "Chikkamagaluru", "Raichur", "Bidar", "Yavatmal", "Chandrapur", "Wardha", "Nanded", "Gondia", "Hingoli", "Parbhani", "LaiLunga",
    "Jalgaon", "Amreli", "Bhuj", "Mehsana", "Anand", "Palanpur", "Surendranagar", "Gandhidham", "Himatnagar", "Junagadh",
    "Porbandar", "Navsari", "Vapi", "Valsad", "Morbi", "Dahod", "Godhra", "Chhapra", "Munger", "Arrah", "Kutta",
    "Begusarai", "Katihar", "Siwan", "Gopalganj", "Samastipur", "Darbhanga", "Sasaram", "Hazaribagh", "Giridih", "Daltonganj", "Chutia"
];  

const CHUNK_SIZE: usize = 1_000_000;
const BUFFER_SIZE: usize = 8 * 1024 * 1024;
const NUM_WORKERS: usize = 10;

struct Timer {
    name: String,
    start: Instant,
}

impl Timer {
    fn new(name: &str) -> Self {
        Timer {
            name: name.to_string(),
            start: Instant::now(),
        }
    }

    fn elapsed(&self) {
        let duration = self.start.elapsed();
        println!("{} took {:.2} seconds", self.name, duration.as_secs_f64());
    }
}

#[derive(Clone)]
struct ChunkData {
    data: String,
    rows: usize,
}

async fn generate_chunk_data(size: usize) -> ChunkData {
    let mut rng = thread_rng();
    let mut data = String::with_capacity(size * 30);
    
    for _ in 0..size {
        let city = CITIES[rng.gen_range(0..CITIES.len())];
        let temp = rng.gen_range(-99.0..99.0);
        data.push_str(&format!("{};{:.1}\n", city, temp));
    }

    ChunkData { data, rows: size }
}

async fn writer_task(
    mut rx: mpsc::Receiver<Option<ChunkData>>,
    output_file: &str,
    total_rows: usize,
) -> std::io::Result<()> {
    let file = File::create(output_file)?;
    
    let mut writer = BufWriter::with_capacity(BUFFER_SIZE, file);
    let mut rows_written = 0;
    let start_time = Instant::now();

    while let Some(Some(chunk)) = rx.recv().await {
        writer.write_all(chunk.data.as_bytes())?;
        rows_written += chunk.rows;

        if rows_written % 50_000_000 == 0 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = rows_written as f64 / elapsed / 1_000_000.0;
            let progress = (rows_written as f64 / total_rows as f64) * 100.0;
            println!("Progress: {:.1}% - Speed: {:.2}M rows/sec", progress, speed);
        }
    }

    writer.flush()?;
    Ok(())
}

pub async fn generate_testcase(num_rows: usize) -> std::io::Result<String> {
    let unique_id = Uuid::new_v4();
    let output_file = format!("testcase_{}_{}.txt", num_rows, unique_id);

    let testcases_dir = "testcases";
    if let Err(e) = std::fs::create_dir_all(testcases_dir) {
        eprintln!("Failed to create testcases directory: {}", e);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create testcases directory: {}", e),
        ));
    }
    
    println!("Starting data generation with {} workers", NUM_WORKERS);
    println!("Output file: {}", output_file);

    let timer = Timer::new("Total execution");
    let gen_timer = Timer::new("Data generation");

    let (tx, rx) = mpsc::channel(NUM_WORKERS * 2);

    let output_file_path = format!("{}/{}", testcases_dir, output_file);
    println!("Writing to: {}", output_file_path);

    let output_file_path_clone = output_file_path.clone();
    let writer_handle = tokio::spawn(async move {
        writer_task(rx, &output_file_path_clone, num_rows).await
    });

    let mut handles = vec![];
    let tx = Arc::new(tx);

    for i in (0..num_rows).step_by(CHUNK_SIZE) {
        let tx = tx.clone();
        let remaining = std::cmp::min(CHUNK_SIZE, num_rows - i);
        
        handles.push(tokio::spawn(async move {
            let chunk = generate_chunk_data(remaining).await;
            tx.send(Some(chunk)).await.unwrap();
        }));
    }

    for handle in handles {
        if let Err(e) = handle.await {
            eprintln!("Task join error: {}", e);
        }
    }

    if let Err(e) = tx.send(None).await {
        eprintln!("Failed to send termination signal: {}", e);
    }

    match writer_handle.await {
        Ok(result) => result?,
        Err(e) => {
            eprintln!("Writer task failed: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
        }
    }

    gen_timer.elapsed();

    match std::fs::metadata(&output_file_path) {
        Ok(metadata) => {
            println!(
                "Final file size: {:.2} GB",
                metadata.len() as f64 / (1024.0 * 1024.0 * 1024.0)
            );
        }
        Err(e) => {
            eprintln!("Failed to get file metadata: {}", e);
        }
    }   

    timer.elapsed();
    Ok(output_file)
}