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

    // Right: Zalo Web — with IPC handler for receiving data from JS
    let sidebar = WebViewBuilder::new()
        .with_url("https://chat.zalo.me")
        .with_user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15")
        .with_ipc_handler(|req| {
            // JS calls: window.ipc.postMessage(JSON.stringify(data))
            let body = req.body();
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(body) {
                *ZALO_MESSAGES.lock().unwrap() = Some(parsed);
            }
        })
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
        .route("/api/zalo/search-and-send", axum::routing::post(zalo_search_and_send_handler))
        .route("/api/zalo/conversations", axum::routing::get(zalo_conversations_handler))
        .route("/api/zalo/messages", axum::routing::get(zalo_messages_handler))
        .route("/api/zalo/_messages_callback", axum::routing::get(zalo_messages_callback));

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

    // Run entire send flow in spawn_blocking to avoid blocking tokio runtime
    let msg = message.clone();
    tokio::task::spawn_blocking(move || {
        // Step 1: Click chat input with REAL OS mouse click via AppleScript
        // JS focus() doesn't move OS cursor — need AppleScript click at coordinates
        // Chat input is at bottom of Zalo sidebar (right side of Haviz window)
        let _ = run_osascript(
            "tell application \"System Events\" to click at {1, 1}"
        ); // dummy click to test permission

        // Get window position, then click bottom of Zalo sidebar
        let pos_result = run_osascript(
            "tell application \"System Events\" to tell process \"haviz_app\" to return {position of window 1, size of window 1}"
        );

        match pos_result {
            Ok(pos_str) => {
                // Parse: {{x, y}, {w, h}}
                let nums: Vec<f64> = pos_str.replace('{', "").replace('}', "")
                    .split(',')
                    .filter_map(|s| s.trim().parse().ok())
                    .collect();
                if nums.len() >= 4 {
                    let (wx, wy, ww, wh) = (nums[0], nums[1], nums[2], nums[3]);
                    // Chat input = bottom of Zalo sidebar (right 400px, bottom ~50px)
                    let click_x = (wx + ww - 200.0) as i64;
                    let click_y = (wy + wh - 60.0) as i64;
                    let _ = run_osascript(&format!(
                        "tell application \"System Events\" to click at {{{}, {}}}", click_x, click_y
                    ));
                }
            }
            Err(_) => {
                // Fallback: JS focus only (might not work)
                eval_zalo_js(r#"(function(){
                    var el=document.querySelector('[contenteditable="true"]');
                    if(el){el.focus();el.click();}
                })();"#).ok();
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(500));

        // Step 2: Clear + type char by char
        let js_type = format!(
            r#"(function(){{
                var el=document.querySelector('[contenteditable="true"]');
                if(!el)return;
                el.focus();
                el.innerHTML='';
                el.dispatchEvent(new InputEvent('input',{{bubbles:true,inputType:'deleteContentBackward'}}));
                var msg="{}";
                for(var i=0;i<msg.length;i++){{
                    var ch=msg[i];
                    el.dispatchEvent(new InputEvent('beforeinput',{{bubbles:true,cancelable:true,inputType:'insertText',data:ch}}));
                    el.innerHTML+=ch;
                    el.dispatchEvent(new InputEvent('input',{{bubbles:true,inputType:'insertText',data:ch}}));
                }}
            }})();"#,
            msg
        );
        eval_zalo_js(&js_type).ok();

        std::thread::sleep(std::time::Duration::from_millis(500));

        // Step 3: Enter + send button
        eval_zalo_js(r#"(function(){
            var el=document.querySelector('[contenteditable="true"]');
            if(!el)return;
            el.focus();
            ['keydown','keypress','keyup'].forEach(function(type){
                el.dispatchEvent(new KeyboardEvent(type,{
                    key:'Enter',code:'Enter',keyCode:13,which:13,
                    bubbles:true,cancelable:true
                }));
            });
            var btns=document.querySelectorAll('[class*="send"],[class*="Send"],button,[role="button"]');
            for(var i=0;i<btns.length;i++){
                var b=btns[i];
                var r=b.getBoundingClientRect();
                if(r.width>10&&r.width<80&&r.height>10&&r.height<80&&r.bottom>window.innerHeight-100){
                    b.click();
                    break;
                }
            }
        })();"#).ok();
    }).await.ok();

    axum::response::Json(serde_json::json!({
        "ok": true,
        "message": req.message,
    }))
}

#[derive(serde::Deserialize)]
struct SearchAndSendRequest {
    to: String,
    message: String,
}

async fn zalo_search_and_send_handler(
    axum::extract::Json(req): axum::extract::Json<SearchAndSendRequest>,
) -> axum::response::Json<serde_json::Value> {
    let to = req.to.replace('\\', "\\\\").replace('"', "\\\"").replace('\'', "\\'");
    let message = req.message.replace('\\', "\\\\").replace('"', "\\\"");

    // Full flow in one spawn_blocking:
    // 1. Search contact (types into search input — this WORKS)
    // 2. Wait for results
    // 3. Enter to open conversation (cursor goes to chat input)
    // 4. Type message (using same InputEvent approach as search)
    // 5. Enter to send

    let result = tokio::task::spawn_blocking(move || {
        // Step 1: Clear search + type contact name (proven approach)
        let js_clear = r#"(function(){
            var inp=document.querySelector('input[type="text"]');
            if(!inp)inp=document.querySelector('input');
            if(!inp)return;
            inp.focus();
            var setter=Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype,'value').set;
            setter.call(inp,'');
            inp.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'deleteContentBackward'}));
        })();"#;
        eval_zalo_js(js_clear).ok();
        std::thread::sleep(std::time::Duration::from_millis(300));

        // Step 2: Type contact name char by char
        let js_search = format!(
            r#"(function(){{
                var inp=document.querySelector('input[type="text"]');
                if(!inp)inp=document.querySelector('input');
                if(!inp)return;
                inp.focus();
                var setter=Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype,'value').set;
                var text='{}';
                for(var i=0;i<text.length;i++){{
                    var ch=text[i];
                    inp.dispatchEvent(new KeyboardEvent('keydown',{{key:ch,bubbles:true}}));
                    setter.call(inp,text.substring(0,i+1));
                    inp.dispatchEvent(new InputEvent('input',{{bubbles:true,data:ch,inputType:'insertText'}}));
                    inp.dispatchEvent(new KeyboardEvent('keyup',{{key:ch,bubbles:true}}));
                }}
            }})();"#,
            to
        );
        eval_zalo_js(&js_search).ok();

        // Step 3: Wait for search results
        std::thread::sleep(std::time::Duration::from_millis(2500));

        // Step 4: Enter to open first result (cursor moves to chat input)
        let js_enter_search = r#"(function(){
            var inp=document.querySelector('input[type="text"]');
            if(!inp)inp=document.querySelector('input');
            if(!inp)return;
            inp.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',code:'Enter',keyCode:13,which:13,bubbles:true}));
            inp.dispatchEvent(new KeyboardEvent('keypress',{key:'Enter',code:'Enter',keyCode:13,which:13,bubbles:true}));
            inp.dispatchEvent(new KeyboardEvent('keyup',{key:'Enter',code:'Enter',keyCode:13,which:13,bubbles:true}));
        })();"#;
        eval_zalo_js(js_enter_search).ok();

        // Step 5: Wait for conversation to open (chat input gets focus)
        std::thread::sleep(std::time::Duration::from_millis(1500));

        // Step 6: Type message into contenteditable chat input
        // After Enter from search, the contenteditable should have focus in the WebView context
        let js_type_msg = format!(
            r#"(function(){{
                var el=document.querySelector('[contenteditable="true"]');
                if(!el)return 'no_input';
                el.focus();
                el.innerHTML='';
                el.dispatchEvent(new InputEvent('input',{{bubbles:true,inputType:'deleteContentBackward'}}));
                var msg="{}";
                for(var i=0;i<msg.length;i++){{
                    var ch=msg[i];
                    el.dispatchEvent(new InputEvent('beforeinput',{{bubbles:true,cancelable:true,inputType:'insertText',data:ch}}));
                    el.innerHTML+=ch;
                    el.dispatchEvent(new InputEvent('input',{{bubbles:true,inputType:'insertText',data:ch}}));
                }}
                return 'typed';
            }})();"#,
            message
        );
        eval_zalo_js(&js_type_msg).ok();
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Step 7: Enter to send + click send button
        eval_zalo_js(r#"(function(){
            var el=document.querySelector('[contenteditable="true"]');
            if(!el)return;
            ['keydown','keypress','keyup'].forEach(function(type){
                el.dispatchEvent(new KeyboardEvent(type,{
                    key:'Enter',code:'Enter',keyCode:13,which:13,
                    bubbles:true,cancelable:true
                }));
            });
            // Click send button
            var btns=document.querySelectorAll('[class*="send"],[class*="Send"],button,[role="button"]');
            for(var i=0;i<btns.length;i++){
                var b=btns[i];
                var r=b.getBoundingClientRect();
                if(r.width>10&&r.width<80&&r.height>10&&r.height<80&&r.bottom>window.innerHeight-100){
                    b.click();break;
                }
            }
        })();"#).ok();

        Ok("sent".to_string())
    }).await.unwrap_or(Err("failed".to_string()));

    axum::response::Json(serde_json::json!({
        "ok": result.is_ok(),
        "to": req.to,
        "message": req.message,
    }))
}

// Shared message buffer — JS in Zalo WebView posts messages here via fetch()
static ZALO_MESSAGES: Mutex<Option<serde_json::Value>> = Mutex::new(None);

async fn zalo_messages_handler() -> axum::response::Json<serde_json::Value> {
    // Clear previous data
    *ZALO_MESSAGES.lock().unwrap() = None;

    // Inject JS → extract messages → send via IPC (window.ipc.postMessage)
    // IPC bypasses CORS entirely — wry built-in mechanism
    let _ = eval_zalo_js(r#"(function(){
        var messages=[];

        // Scan all leaf text nodes deeply nested (chat content)
        var all=document.querySelectorAll('*');
        for(var i=0;i<all.length;i++){
            var el=all[i];
            if(el.children.length>3)continue;
            var text=el.textContent?el.textContent.trim():'';
            if(text.length<1||text.length>500)continue;
            if(el.children.length>0){
                var childText='';
                for(var c=0;c<el.children.length;c++)childText+=(el.children[c].textContent||'');
                if(childText.trim()===text)continue;
            }
            var cls=(typeof el.className==='string')?el.className:'';
            var depth=0;var p=el;
            while(p&&p!==document.body){depth++;p=p.parentElement;}
            if(depth<8)continue;
            messages.push({
                content:text,
                class:cls.substring(0,60),
                tag:el.tagName,
                depth:depth
            });
        }

        // Send via wry IPC — no CORS issues
        if(window.ipc&&window.ipc.postMessage){
            window.ipc.postMessage(JSON.stringify(messages.slice(-100)));
        }
    })();"#);

    // Wait for IPC callback
    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if ZALO_MESSAGES.lock().unwrap().is_some() {
            break;
        }
    }

    let data = ZALO_MESSAGES.lock().unwrap().take();
    match data {
        Some(msgs) => axum::response::Json(serde_json::json!({
            "ok": true,
            "messages": msgs,
        })),
        None => axum::response::Json(serde_json::json!({
            "ok": false,
            "messages": [],
            "note": "No messages extracted. Open a conversation in Zalo sidebar first.",
        })),
    }
}

// Internal callback — receives messages via GET (image src trick) or POST
async fn zalo_messages_callback(
    query: axum::extract::Query<std::collections::HashMap<String, String>>,
) -> axum::response::Response {
    if let Some(data) = query.get("data") {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
            *ZALO_MESSAGES.lock().unwrap() = Some(parsed);
        }
    }
    // Return 1x1 transparent PNG (for image src)
    axum::response::Response::builder()
        .header("Content-Type", "image/png")
        .body(axum::body::Body::from(vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
            0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00,
            0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x62, 0x00, 0x00, 0x00, 0x02,
            0x00, 0x01, 0xE5, 0x27, 0xDE, 0xFC, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
            0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ]))
        .unwrap()
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
