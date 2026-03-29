use std::fs;
use std::io::Write;

pub fn set_error_with_details(prefix: &str, stderr: &str) -> String {
    let first_line = stderr.lines().next().unwrap_or(stderr);
    let truncated = if first_line.len() > 80 {
        format!("{}...", &first_line[..77])
    } else {
        first_line.to_string()
    };

    if let Some(path) = save_error_to_file(stderr) {
        format!("{prefix}: {truncated} (full error: {path})")
    } else {
        format!("{prefix}: {truncated}")
    }
}

fn save_error_to_file(error: &str) -> Option<String> {
    let temp_dir = std::env::temp_dir();
    let error_file = temp_dir.join(format!("jju-error-{}.log", std::process::id()));
    let path = error_file.to_string_lossy().to_string();

    match fs::File::create(&error_file) {
        Ok(mut file) => {
            if file.write_all(error.as_bytes()).is_ok() {
                Some(path)
            } else {
                None
            }
        }
        Err(_) => None,
    }
}
