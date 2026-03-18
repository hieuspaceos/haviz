use std::process::Command;

/// Invoke the compiled Swift helper to read Zalo messages via AX API.
/// Returns raw JSON string on success.
pub fn read_zalo_messages(reader_path: &str) -> Result<String, String> {
    let output = Command::new(reader_path)
        .output()
        .map_err(|e| format!("Failed to run zalo_reader: {}. Is it compiled? Run: make helpers", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Zalo not open is a common non-error
        if stderr.contains("Zalo chưa mở") || stderr.contains("not running") {
            return Err("zalo_not_running".to_string());
        }
        return Err(format!("zalo_reader failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        return Err("zalo_reader returned empty output".to_string());
    }

    Ok(stdout.to_string())
}
