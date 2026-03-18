/// Haviz App — Dashboard + Zalo Web Sidebar
///
/// Layout:
/// ┌──────────────────────────┬─────────────┐
/// │    Haviz Dashboard       │  Zalo Web   │
/// │    (localhost:9999)      │  Sidebar    │
/// │    Inbox, AI Drafts      │  (~400px)   │
/// └──────────────────────────┴─────────────┘

use std::sync::{Arc, Mutex};
use tao::dpi::{LogicalPosition, LogicalSize};
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::WindowBuilder;
use wry::{Rect, WebView, WebViewBuilder};
#[cfg(target_os = "macos")]
use wry::WebViewBuilderExtDarwin;

const WINDOW_W: f64 = 1400.0;
const WINDOW_H: f64 = 900.0;
const SIDEBAR_W: f64 = 400.0;

/// Shared state: allows Agent server to execute JS on the Zalo sidebar WebView
static ZALO_JS_QUEUE: Mutex<Vec<(String, std::sync::mpsc::Sender<String>)>> = Mutex::new(Vec::new());

/// Queue a JS evaluation on the Zalo sidebar, returns result
pub fn eval_zalo_js(js: &str) -> Result<String, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    {
        let mut queue = ZALO_JS_QUEUE.lock().unwrap();
        queue.push((js.to_string(), tx));
    }
    rx.recv_timeout(std::time::Duration::from_secs(10))
        .map_err(|e| format!("JS eval timeout: {}", e))
}

#[derive(Clone)]
enum UserEvent {
    ProcessJsQueue,
}

fn main() {
    println!("╔═══════════════════════════════════════╗");
    println!("║  Haviz — Revenue Intelligence         ║");
    println!("╚═══════════════════════════════════════╝\n");

    // Start Agent HTTP server in background
    let _agent = std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(start_agent());
    });

    std::thread::sleep(std::time::Duration::from_millis(500));

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title("Haviz")
        .with_inner_size(LogicalSize::new(WINDOW_W, WINDOW_H))
        .build(&event_loop)
        .expect("Failed to create window");

    let store_id: [u8; 16] = *b"haviz_zalo_web_1";

    // Left: Dashboard — use Vite dev server if running, otherwise Agent static
    let dashboard_url = if reqwest::blocking::get("http://localhost:3000").is_ok() {
        "http://localhost:3000" // Vite dev server (hot reload)
    } else {
        "http://localhost:9999" // Production (built static files)
    };
    println!("Dashboard: {}", dashboard_url);

    let _dashboard = WebViewBuilder::new()
        .with_url(dashboard_url)
        .with_bounds(Rect {
            position: LogicalPosition::new(0.0, 0.0).into(),
            size: LogicalSize::new(WINDOW_W - SIDEBAR_W, WINDOW_H).into(),
        })
        .with_devtools(true)
        .build_as_child(&window)
        .expect("Failed to create dashboard");

    // Right: Zalo Web
    let sidebar = WebViewBuilder::new()
        .with_url("https://chat.zalo.me")
        .with_user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15")
        .with_bounds(Rect {
            position: LogicalPosition::new(WINDOW_W - SIDEBAR_W, 0.0).into(),
            size: LogicalSize::new(SIDEBAR_W, WINDOW_H).into(),
        })
        .with_incognito(false)
        .with_data_store_identifier(store_id)
        .with_devtools(true)
        .build_as_child(&window)
        .expect("Failed to create Zalo sidebar");

    println!("✅ Haviz đang chạy!");
    println!("   Dashboard: http://localhost:9999");
    println!("   Zalo Web sidebar (phải)\n");

    // Poll JS queue periodically
    let proxy2 = proxy.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
            let _ = proxy2.send_event(UserEvent::ProcessJsQueue);
        }
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvent::ProcessJsQueue) => {
                // Process pending JS evaluations on sidebar
                let mut queue = ZALO_JS_QUEUE.lock().unwrap();
                for (js, tx) in queue.drain(..) {
                    match sidebar.evaluate_script(&js) {
                        Ok(()) => {
                            // wry evaluate_script doesn't return value directly
                            // We use a workaround: JS sets window.__haviz_result
                            let _ = tx.send("ok".to_string());
                        }
                        Err(e) => {
                            let _ = tx.send(format!("error:{}", e));
                        }
                    }
                }
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(new_size) => {
                    let w = new_size.width as f64;
                    let h = new_size.height as f64;
                    let _ = _dashboard.set_bounds(Rect {
                        position: LogicalPosition::new(0.0, 0.0).into(),
                        size: LogicalSize::new(w - SIDEBAR_W, h).into(),
                    });
                    let _ = sidebar.set_bounds(Rect {
                        position: LogicalPosition::new(w - SIDEBAR_W, 0.0).into(),
                        size: LogicalSize::new(SIDEBAR_W, h).into(),
                    });
                }
                _ => {}
            },
            _ => {}
        }
    });
}

async fn start_agent() {
    // Load .env.local
    let env_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(".env.local");
    if env_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&env_path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') { continue; }
                if let Some((key, val)) = line.split_once('=') {
                    std::env::set_var(key.trim(), val.trim());
                }
            }
        }
    }

    let config = haviz_agent::config::Config::load();
    let db = std::sync::Arc::new(
        haviz_agent::db::Database::open(&config.db_path).expect("Failed to open DB"),
    );

    // Extended router with Zalo control + screenshot endpoints
    let app = haviz_agent::server::create_router(db)
        .route("/api/screenshot", axum::routing::get(screenshot_handler))
        .route("/api/zalo/search", axum::routing::post(zalo_search_handler))
        .route("/api/zalo/open", axum::routing::post(zalo_open_handler))
        .route("/api/zalo/send", axum::routing::post(zalo_send_handler))
        .route("/api/zalo/conversations", axum::routing::get(zalo_conversations_handler));

    let addr = format!("0.0.0.0:{}", config.http_port);
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}

// === Screenshot ===
// Requires: System Settings → Privacy & Security → Screen Recording → enable Terminal
async fn screenshot_handler() -> axum::response::Response {
    use std::process::Command;
    let path = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("Haviz")
        .join("screenshot.png");

    // screencapture -x = no sound, captures the screen
    // -w = interactive window select (but we want automatic)
    let _ = Command::new("screencapture")
        .args(["-x", "-o", path.to_str().unwrap()])
        .output();

    match std::fs::read(&path) {
        Ok(bytes) => axum::response::Response::builder()
            .header("Content-Type", "image/png")
            .body(axum::body::Body::from(bytes))
            .unwrap(),
        Err(e) => axum::response::Response::builder()
            .status(500)
            .header("Content-Type", "text/plain")
            .body(axum::body::Body::from(format!(
                "Screenshot failed: {}. Add Terminal to Screen Recording in System Settings.", e
            )))
            .unwrap(),
    }
}

// === Zalo Control via AppleScript ===
// All interactions use OS-level AppleScript (click coordinates, clipboard paste)
// This is the SAME approach validated in agent-prototype.js — works with Zalo's React events

fn run_osascript(script: &str) -> Result<String, String> {
    // osascript -e only supports single-line. For multiline, use temp file.
    let output = if script.contains('\n') {
        let tmp = std::env::temp_dir().join("haviz_osa.scpt");
        std::fs::write(&tmp, script).map_err(|e| e.to_string())?;
        let o = std::process::Command::new("osascript").arg(&tmp).output();
        let _ = std::fs::remove_file(&tmp);
        o
    } else {
        std::process::Command::new("osascript").arg("-e").arg(script).output()
    }
    .map_err(|e| format!("osascript failed: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(err);
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[derive(serde::Deserialize)]
struct SearchRequest { query: String }

#[derive(serde::Deserialize)]
struct OpenRequest { index: usize }

#[derive(serde::Deserialize)]
struct SendMsgRequest { message: String }

async fn zalo_search_handler(
    axum::extract::Json(req): axum::extract::Json<SearchRequest>,
) -> axum::response::Json<serde_json::Value> {
    let query = req.query.clone();
    let auto_enter = !req.query.is_empty(); // auto-open first result

    // Step 1: Focus input, clear, set value
    let js_type = format!(
        r#"(function(){{
            var inp=document.querySelector('input[type="text"]');
            if(!inp)inp=document.querySelector('input');
            if(!inp)return;
            inp.focus();
            inp.value='';
            inp.dispatchEvent(new Event('input',{{bubbles:true}}));
        }})();"#
    );
    let _ = eval_zalo_js(&js_type);
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Step 2: Simulate real keyboard typing using InputEvent + KeyboardEvent
    // Zalo React listens for these specific events, not just input.value change
    let js_set = format!(
        r#"(function(){{
            var inp=document.querySelector('input[type="text"]');
            if(!inp)inp=document.querySelector('input');
            if(!inp)return;
            inp.focus();
            inp.value='';
            // Use native input setter to bypass React's synthetic events
            var nativeInputValueSetter=Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype,'value').set;
            var text='{}';
            for(var i=0;i<text.length;i++){{
                var char=text[i];
                // KeyboardEvent for keydown
                inp.dispatchEvent(new KeyboardEvent('keydown',{{key:char,code:'Key'+char.toUpperCase(),bubbles:true}}));
                // Set value using native setter (React detects this)
                nativeInputValueSetter.call(inp,text.substring(0,i+1));
                // InputEvent (React uses this for controlled inputs)
                inp.dispatchEvent(new InputEvent('input',{{bubbles:true,data:char,inputType:'insertText'}}));
                // KeyboardEvent for keyup
                inp.dispatchEvent(new KeyboardEvent('keyup',{{key:char,code:'Key'+char.toUpperCase(),bubbles:true}}));
            }}
        }})();"#,
        query.replace('\\', "\\\\").replace('"', "\\\"").replace('\'', "\\'")
    );
    let _ = eval_zalo_js(&js_set);

    // Step 3: Wait for Zalo to process search
    std::thread::sleep(std::time::Duration::from_millis(2500));

    // Step 4: Press Enter to open first result
    if auto_enter {
        let js_enter = r#"(function(){
            var inp=document.querySelector('input[type="text"]');
            if(!inp)inp=document.querySelector('input');
            if(!inp)return;
            inp.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',code:'Enter',keyCode:13,which:13,bubbles:true}));
            inp.dispatchEvent(new KeyboardEvent('keypress',{key:'Enter',code:'Enter',keyCode:13,which:13,bubbles:true}));
            inp.dispatchEvent(new KeyboardEvent('keyup',{key:'Enter',code:'Enter',keyCode:13,which:13,bubbles:true}));
        })();"#;
        let _ = eval_zalo_js(js_enter);
    }

    axum::response::Json(serde_json::json!({
        "ok": true,
        "query": req.query,
        "auto_enter": auto_enter,
    }))
}

async fn zalo_open_handler(
    axum::extract::Json(req): axum::extract::Json<OpenRequest>,
) -> axum::response::Json<serde_json::Value> {
    let idx = req.index.saturating_sub(1);

    // JS: find clickable items, simulate mousedown+mouseup+click on the Nth one
    let js = format!(
        r#"(function(){{
            var items=document.querySelectorAll('[class*="conv-item"],[class*="contact-item"],[class*="chat-item"]');
            if(items.length===0){{
                // Fallback: find parent elements of text nodes in the list area
                var all=document.querySelectorAll('*');
                var clickable=[];
                for(var i=0;i<all.length;i++){{
                    var el=all[i];
                    if(el.children.length>0 && el.children.length<10 && el.offsetHeight>40 && el.offsetHeight<100){{
                        var hasText=el.querySelector('span,div');
                        if(hasText && hasText.textContent.trim().length>1){{
                            clickable.push(el);
                        }}
                    }}
                }}
                items=clickable;
            }}
            if({idx}<items.length){{
                var target=items[{idx}];
                target.scrollIntoView({{block:'center'}});
                var rect=target.getBoundingClientRect();
                var x=rect.x+rect.width/2;
                var y=rect.y+rect.height/2;
                // Dispatch real mouse events at element center
                var opts={{bubbles:true,clientX:x,clientY:y}};
                target.dispatchEvent(new MouseEvent('mousedown',opts));
                target.dispatchEvent(new MouseEvent('mouseup',opts));
                target.dispatchEvent(new MouseEvent('click',opts));
                return 'clicked';
            }}
            return 'not_found:'+items.length+' items';
        }})();"#,
        idx = idx
    );
    let _ = eval_zalo_js(&js);

    axum::response::Json(serde_json::json!({
        "ok": true,
        "clicked_index": req.index,
    }))
}

async fn zalo_send_handler(
    axum::extract::Json(req): axum::extract::Json<SendMsgRequest>,
) -> axum::response::Json<serde_json::Value> {
    let message = req.message.replace('\\', "\\\\").replace('"', "\\\"");

    // Step 1: Click on chat input to focus it
    let _ = eval_zalo_js(r#"(function(){
        var el=document.querySelector('[contenteditable="true"]');
        if(!el)return 'not_found';
        el.focus();
        el.click();
        // Also dispatch mouse events for React
        var rect=el.getBoundingClientRect();
        var opts={bubbles:true,clientX:rect.x+10,clientY:rect.y+10};
        el.dispatchEvent(new MouseEvent('mousedown',opts));
        el.dispatchEvent(new MouseEvent('mouseup',opts));
        el.dispatchEvent(new MouseEvent('click',opts));
        return 'focused';
    })();"#);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Step 2: Type text character by character using InputEvent
    // Same nativeInputValueSetter trick won't work for contenteditable.
    // For contenteditable, dispatch beforeinput + input events per character.
    let js_type = format!(
        r#"(function(){{
            var el=document.querySelector('[contenteditable="true"]');
            if(!el)return 'not_found';
            el.focus();

            // Clear existing content
            el.innerHTML='';
            el.dispatchEvent(new InputEvent('input',{{bubbles:true,inputType:'deleteContentBackward'}}));

            // Type text char by char
            var text="{}";
            for(var i=0;i<text.length;i++){{
                var ch=text[i];
                el.dispatchEvent(new InputEvent('beforeinput',{{bubbles:true,cancelable:true,inputType:'insertText',data:ch}}));
                el.innerHTML+=ch;
                el.dispatchEvent(new InputEvent('input',{{bubbles:true,inputType:'insertText',data:ch}}));
            }}
            return 'typed:'+text.length;
        }})();"#,
        message
    );
    let _ = eval_zalo_js(&js_type);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Step 3: Send — try multiple methods
    let _ = eval_zalo_js(r#"(function(){
        var el=document.querySelector('[contenteditable="true"]');
        if(!el)return;

        // Method 1: Enter key with all event types
        ['keydown','keypress','keyup'].forEach(function(type){
            el.dispatchEvent(new KeyboardEvent(type,{
                key:'Enter',code:'Enter',keyCode:13,which:13,
                bubbles:true,cancelable:true
            }));
        });

        // Method 2: Find and click send button (Zalo has an icon button)
        var allEls=document.querySelectorAll('[class*="send"],[class*="Send"],button,[role="button"]');
        for(var i=0;i<allEls.length;i++){
            var b=allEls[i];
            var rect=b.getBoundingClientRect();
            // Send button is usually small (icon), near bottom-right of chat
            if(rect.width>10 && rect.width<80 && rect.height>10 && rect.height<80
               && rect.bottom>window.innerHeight-100){
                b.click();
                b.dispatchEvent(new MouseEvent('click',{bubbles:true}));
                return 'clicked_btn';
            }
        }

        // Method 3: Submit form if exists
        var form=el.closest('form');
        if(form){form.submit();return 'form_submit';}

        return 'enter_only';
    })();"#);

    axum::response::Json(serde_json::json!({
        "ok": true,
        "message": req.message,
    }))
}

async fn zalo_conversations_handler() -> axum::response::Json<serde_json::Value> {
    // Use AX API to read conversations from the Haviz WebView
    let reader_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("helpers").join("zalo_web_reader");

    let result = tokio::task::spawn_blocking(move || {
        std::process::Command::new(&reader_path).arg("haviz_app").output()
    }).await;

    match result {
        Ok(Ok(output)) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            match serde_json::from_str::<serde_json::Value>(&stdout) {
                Ok(data) => axum::response::Json(data),
                Err(_) => axum::response::Json(serde_json::json!({"ok": false, "error": "parse_error"})),
            }
        }
        _ => {
            // AX API might not find "haviz_app" as browser name
            // Fallback: return empty with helpful message
            axum::response::Json(serde_json::json!({
                "ok": false,
                "conversations": [],
                "note": "AX API reader could not find Haviz WebView. Conversations visible in sidebar.",
            }))
        }
    }
}
