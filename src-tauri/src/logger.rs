use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use once_cell::sync::Lazy;

static LOG_FILE: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));

fn get_log_dir() -> PathBuf {
    let home = dirs_next::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".codeatlas-studio").join("logs")
}

fn timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let millis = now.subsec_millis();

    // Convert to local time (simple UTC+8 for China, or use system offset)
    let total_secs = secs + 8 * 3600; // UTC+8, adjust as needed
    let days = total_secs / 86400;
    let remaining = total_secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    // Simple date calculation (approximate)
    let year = 1970 + days / 365;
    let day_of_year = days % 365;
    let month = (day_of_year / 30) + 1;
    let day = (day_of_year % 30) + 1;

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
        year, month, day, hours, minutes, seconds, millis
    )
}

fn get_log_file_path() -> PathBuf {
    let mut cached = LOG_FILE.lock().unwrap();
    if let Some(ref path) = *cached {
        return path.clone();
    }

    let log_dir = get_log_dir();
    let _ = fs::create_dir_all(&log_dir);

    // Log file named by date
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let days = (now.as_secs() + 8 * 3600) / 86400;
    let year = 1970 + days / 365;
    let day_of_year = days % 365;
    let month = (day_of_year / 30) + 1;
    let day = (day_of_year % 30) + 1;

    let filename = format!("app-{:04}-{:02}-{:02}.log", year, month, day);
    let path = log_dir.join(filename);
    *cached = Some(path.clone());
    path
}

pub fn log(level: &str, target: &str, message: &str) {
    let path = get_log_file_path();
    let ts = timestamp();
    let line = format!("[{}] [{:<5}] [{}] {}\n", ts, level, target, message);

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = file.write_all(line.as_bytes());
    }

    // Also print to stderr for debug builds
    #[cfg(debug_assertions)]
    eprint!("{}", line);
}

#[macro_export]
macro_rules! log_info {
    ($target:expr, $($arg:tt)*) => {
        $crate::logger::log("INFO", $target, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($target:expr, $($arg:tt)*) => {
        $crate::logger::log("WARN", $target, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($target:expr, $($arg:tt)*) => {
        $crate::logger::log("ERROR", $target, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($target:expr, $($arg:tt)*) => {
        $crate::logger::log("DEBUG", $target, &format!($($arg)*))
    };
}

/// Get the current log file path for exposing to frontend
pub fn get_log_path() -> String {
    get_log_file_path().to_string_lossy().to_string()
}

/// Get recent log lines (last N lines)
pub fn get_recent_logs(n: usize) -> Vec<String> {
    let path = get_log_file_path();
    let content = fs::read_to_string(&path).unwrap_or_default();
    let lines: Vec<&str> = content.lines().collect();
    let start = if lines.len() > n { lines.len() - n } else { 0 };
    lines[start..].iter().map(|s| s.to_string()).collect()
}
