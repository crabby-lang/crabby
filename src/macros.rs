use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

#[macro_export]
macro_rules! crabby {
    ($($arg:tt)*) => {
        {

            // Get the current timestamp
            let start = SystemTime::now();
            let since_the_epoch = start.duration_since(UNIX_EPOCH)
                .expect("Time went backwards");
            let timestamp = since_the_epoch.as_secs();

            // Print to console
            println!("[{}] {}", timestamp, format!($($arg)*));

            // Log to a file
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open("crabby.log")
                .expect("Unable to open log file");
            writeln!(file, "[{}] {}", timestamp, format!($($arg)*))
                .expect("Unable to write to log file");
        }
    };
}
