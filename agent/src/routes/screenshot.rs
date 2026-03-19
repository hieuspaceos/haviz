/// Route handler for GET /api/screenshot — captures screen via OS-native tool.
///
/// macOS: `screencapture -x -o <path>` (requires Screen Recording permission).
/// Windows: PowerShell `[System.Windows.Forms.Screen]` + `System.Drawing.Bitmap`.
use axum::{body::Body, response::Response};
use std::path::PathBuf;
use std::process::Command;

fn screenshot_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Haviz")
        .join("screenshot.png")
}

/// GET /api/screenshot
pub async fn screenshot_handler() -> Response {
    let path = screenshot_path();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let capture_ok = capture_screen(&path);

    if !capture_ok {
        return Response::builder()
            .status(500)
            .header("Content-Type", "text/plain")
            .body(Body::from(capture_error_hint()))
            .unwrap();
    }

    match std::fs::read(&path) {
        Ok(bytes) => Response::builder()
            .header("Content-Type", "image/png")
            .body(Body::from(bytes))
            .unwrap(),
        Err(e) => Response::builder()
            .status(500)
            .header("Content-Type", "text/plain")
            .body(Body::from(format!("Screenshot read failed: {}", e)))
            .unwrap(),
    }
}

/// Capture full screen to `path`. Returns true on success.
fn capture_screen(path: &PathBuf) -> bool {
    #[cfg(target_os = "macos")]
    {
        // -x = no shutter sound, -o = no shadow
        Command::new("screencapture")
            .args(["-x", "-o", path.to_str().unwrap_or("screenshot.png")])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    #[cfg(target_os = "windows")]
    {
        // PowerShell one-liner: capture primary screen to PNG
        let ps_script = format!(
            r#"Add-Type -AssemblyName System.Windows.Forms,System.Drawing;
$s=[System.Windows.Forms.Screen]::PrimaryScreen.Bounds;
$bmp=New-Object System.Drawing.Bitmap($s.Width,$s.Height);
$g=[System.Drawing.Graphics]::FromImage($bmp);
$g.CopyFromScreen($s.Left,$s.Top,0,0,$s.Size);
$bmp.Save('{}');
$g.Dispose();$bmp.Dispose()"#,
            path.to_str().unwrap_or("screenshot.png").replace('\'', "''")
        );

        Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &ps_script])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        false
    }
}

fn capture_error_hint() -> String {
    #[cfg(target_os = "macos")]
    return "Screenshot failed. Add Terminal to Screen Recording in System Settings.".to_string();

    #[cfg(target_os = "windows")]
    return "Screenshot failed. Ensure PowerShell and System.Windows.Forms are available.".to_string();

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    return "Screenshot not supported on this platform.".to_string();
}
