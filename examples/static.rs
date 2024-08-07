use configparser::ini::Ini;

fn main() {
    let mut config = Ini::new();
    // config.set("CS", "timestamp", Some(format!("{}",chrono::Utc::now().timestamp_millis())));
    // config.write("orkz.ini").unwrap();
    let map = match config.load("orkz.ini") {
        Ok(map) => map,
        Err(e) => {
            println!("Error: {:?}", e);
            return;
        }
    };
    println!("{:?}", map);
}