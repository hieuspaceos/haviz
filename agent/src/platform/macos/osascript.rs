/// Low-level osascript runner used by Zalo control handlers.
/// Supports both single-line (-e) and multi-line (temp file) scripts.

/// Execute an AppleScript snippet, returning trimmed stdout on success.
/// Multi-line scripts are written to a temp file to avoid shell quoting issues.
pub fn run_osascript(script: &str) -> Result<String, String> {
    let output = if script.contains('\n') {
        // osascript -e does not support multi-line; use temp file instead
        let tmp = std::env::temp_dir().join("haviz_osa.scpt");
        std::fs::write(&tmp, script).map_err(|e| e.to_string())?;
        let o = std::process::Command::new("osascript").arg(&tmp).output();
        let _ = std::fs::remove_file(&tmp);
        o
    } else {
        std::process::Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
    }
    .map_err(|e| format!("osascript failed: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(err);
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
