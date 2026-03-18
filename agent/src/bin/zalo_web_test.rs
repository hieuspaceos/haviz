/// Haviz — Zalo Web Interactive Test
/// Chạy: cargo run --bin zalo_web_test
///
/// Flow:
/// 1. Mở Chrome (có GUI) → chat.zalo.me
/// 2. Bạn scan QR code trên điện thoại
/// 3. Agent detect login → save cookies
/// 4. Liệt kê danh bạ / conversations
/// 5. Tìm kiếm user theo tên
/// 6. Đọc tin nhắn conversation đang mở

use headless_chrome::{Browser, LaunchOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;

const ZALO_URL: &str = "https://chat.zalo.me";

fn main() {
    println!("╔══════════════════════════════════════════╗");
    println!("║  Haviz — Zalo Web Interactive Test       ║");
    println!("╚══════════════════════════════════════════╝\n");

    // Find chrome
    let chrome_path = std::env::var("HAVIZ_CHROME_PATH").ok().map(PathBuf::from);
    if let Some(ref p) = chrome_path {
        println!("Chrome path: {}", p.display());
    } else {
        println!("Chrome path: auto-detect (system Chrome)");
    }

    // Persistent Chrome profile — saves cookies, localStorage, IndexedDB, sessions
    let profile_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Haviz")
        .join("chrome-profile");
    std::fs::create_dir_all(&profile_dir).ok();

    let first_run = !profile_dir.join("Default").exists();
    println!("Chrome profile: {}", profile_dir.display());
    println!("First run: {} ({})\n", first_run,
        if first_run { "sẽ cần scan QR" } else { "session đã lưu, có thể skip QR" });

    // Launch Chrome — VISIBLE, with persistent profile
    println!("Launching Chrome (visible mode, persistent profile)...");
    let mut builder = LaunchOptions::default_builder();
    builder
        .headless(false)
        .sandbox(false)
        .idle_browser_timeout(Duration::from_secs(600))
        .window_size(Some((1200, 800)))
        .user_data_dir(Some(profile_dir.clone()));

    if let Some(ref p) = chrome_path {
        builder.path(Some(p.clone()));
    }

    let browser = match Browser::new(builder.build().unwrap()) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to launch Chrome: {}", e);
            eprintln!("Tip: set HAVIZ_CHROME_PATH to chrome or chrome-headless-shell path");
            return;
        }
    };

    // Use existing tab or create new one
    let tab = {
        let tabs = browser.get_tabs().lock().unwrap();
        if let Some(t) = tabs.first() {
            t.clone()
        } else {
            drop(tabs);
            match browser.new_tab() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Failed to create tab: {}", e);
                    return;
                }
            }
        }
    };

    // Navigate
    println!("Navigating to {}...", ZALO_URL);
    if let Err(e) = tab.navigate_to(ZALO_URL) {
        eprintln!("Navigation failed: {}", e);
        return;
    }
    let _ = tab.wait_until_navigated();
    println!("Đợi Zalo load (5s)...");
    std::thread::sleep(Duration::from_secs(5));

    // Check login state
    let is_logged_in = check_logged_in(&tab);

    if !is_logged_in {
        println!("\n┌────────────────────────────────────────────┐");
        println!("│  Chưa đăng nhập. Hãy SCAN QR CODE          │");
        println!("│  trên cửa sổ Chrome vừa mở!                │");
        println!("│                                              │");
        println!("│  Đang chờ... (tối đa 120 giây)              │");
        println!("└────────────────────────────────────────────┘\n");

        let mut logged_in = false;
        for i in 1..=40 {
            std::thread::sleep(Duration::from_secs(3));
            if check_logged_in(&tab) {
                logged_in = true;
                println!("\n✅ Đăng nhập thành công!");
                break;
            }
            if i % 5 == 0 {
                println!("  Đang chờ QR scan... ({}s)", i * 3);
            }
        }

        if !logged_in {
            eprintln!("❌ Timeout — không detect được đăng nhập.");
            eprintln!("   Nhấn Enter để tiếp tục kiểm tra thủ công...");
            let _ = read_line();
        }
    } else {
        println!("✅ Đã đăng nhập (cookies còn hiệu lực)!");
    }

    // Session auto-saved by Chrome profile — no manual cookie save needed
    println!("\n✅ Session tự động lưu trong Chrome profile.");
    println!("   Lần chạy sau sẽ không cần scan QR lại.\n");

    // Interactive menu
    loop {
        println!("\n╔══════════════════════════════════════════╗");
        println!("║  Chọn thao tác:                          ║");
        println!("║  1. Xem danh sách conversations          ║");
        println!("║  2. Tìm kiếm user                        ║");
        println!("║  3. Đọc tin nhắn conversation đang mở    ║");
        println!("║  4. Xem thông tin trang hiện tại          ║");
        println!("║  5. Chụp screenshot                       ║");
        println!("║  0. Thoát                                ║");
        println!("╚══════════════════════════════════════════╝");
        print!("> ");
        io::stdout().flush().ok();

        let input = read_line();
        match input.trim() {
            "1" => list_conversations(&tab),
            "2" => search_user(&tab),
            "3" => read_messages(&tab),
            "4" => page_info(&tab),
            "5" => take_screenshot(&tab, &profile_dir),
            "0" | "q" | "quit" | "exit" => {
                println!("Bye!");
                break;
            }
            _ => println!("Lựa chọn không hợp lệ."),
        }
    }
}

fn check_logged_in(tab: &headless_chrome::Tab) -> bool {
    // Multiple signals: .truncate = contact names, #chatArea, conversation list
    let js = r#"
    (() => {
        const signals = [
            document.querySelector('.truncate'),
            document.querySelector('[class*="conv-item"]'),
            document.querySelector('[class*="sidebar"]'),
            document.querySelector('[class*="chat-input"]'),
            document.querySelector('[contenteditable="true"]'),
        ];
        return signals.some(s => s !== null);
    })()
    "#;
    tab.evaluate(js, false)
        .map(|r| r.value.as_ref().and_then(|v| v.as_bool()).unwrap_or(false))
        .unwrap_or(false)
}

fn list_conversations(tab: &headless_chrome::Tab) {
    println!("\n📋 Đang lấy danh sách conversations...\n");

    let count = list_clickable_items(tab, "Conversations");
    if count == 0 { return; }

    // Ask user to click one
    print!("\n  Nhập số để mở conversation (hoặc Enter để bỏ qua): ");
    io::stdout().flush().ok();
    let choice = read_line();
    let choice = choice.trim();
    if choice.is_empty() { return; }

    if let Ok(idx) = choice.parse::<usize>() {
        click_item_by_index(tab, idx);
    }
}

fn search_user(tab: &headless_chrome::Tab) {
    print!("\n🔍 Nhập tên cần tìm: ");
    io::stdout().flush().ok();
    let query = read_line();
    let query = query.trim().to_string();
    if query.is_empty() {
        println!("  Bỏ qua.");
        return;
    }

    println!("  Đang tìm '{}'...", query);

    // Use CDP keyboard to type into search — more reliable than setting .value
    // Step 1: Click on search area to focus
    let focus_js = r#"
    (() => {
        const inputs = document.querySelectorAll('input[type="text"], input[placeholder*="Tìm"], input[placeholder*="tìm"], input[placeholder*="Search"]');
        for (const inp of inputs) {
            if (inp.offsetParent !== null) {
                inp.focus();
                inp.click();
                // Clear existing text
                inp.value = '';
                inp.dispatchEvent(new Event('input', { bubbles: true }));
                return JSON.stringify({ ok: true, placeholder: inp.placeholder || inp.className });
            }
        }
        // Fallback
        const inp = document.querySelector('input:not([type="hidden"])');
        if (inp) {
            inp.focus();
            inp.click();
            inp.value = '';
            inp.dispatchEvent(new Event('input', { bubbles: true }));
            return JSON.stringify({ ok: true, placeholder: inp.placeholder || 'unknown' });
        }
        return JSON.stringify({ ok: false, error: "Search input not found" });
    })()
    "#;

    match tab.evaluate(focus_js, false) {
        Ok(r) => {
            let json_str = r.value.as_ref().and_then(|v| v.as_str()).unwrap_or("{}");
            let result: serde_json::Value = serde_json::from_str(json_str).unwrap_or_default();
            if result["ok"].as_bool() != Some(true) {
                println!("  ❌ {}", result["error"].as_str().unwrap_or("Search input not found"));
                return;
            }
            println!("  Focused search: {}", result["placeholder"]);
        }
        Err(e) => { eprintln!("  ❌ {}", e); return; }
    }

    std::thread::sleep(Duration::from_millis(300));

    // Step 2: Type each character via CDP (handles Vietnamese input correctly)
    for ch in query.chars() {
        let _ = tab.press_key(&ch.to_string());
        std::thread::sleep(Duration::from_millis(50));
    }

    println!("  Typed '{}'. Đợi kết quả (2s)...", query);
    std::thread::sleep(Duration::from_secs(2));

    // Step 3: Read search results
    let count = get_clickable_items_count(tab);
    if count == 0 {
        println!("  Không tìm thấy kết quả.");
        return;
    }

    list_clickable_items(tab, "Kết quả tìm kiếm");

    // Step 4: Ask user which to click
    print!("\n  Nhập số để chọn (hoặc Enter để bỏ qua): ");
    io::stdout().flush().ok();
    let choice = read_line();
    let choice = choice.trim();
    if choice.is_empty() { return; }

    if let Ok(idx) = choice.parse::<usize>() {
        click_item_by_index(tab, idx);
    }
}

// === Shared helpers for listing & clicking items ===

fn get_clickable_items_count(tab: &headless_chrome::Tab) -> usize {
    let js = r#"
    (() => {
        const items = document.querySelectorAll(
            '[class*="conv-item"], [class*="contact-item"], [class*="chat-item"],
             [class*="thread"], [class*="search-result"], [class*="suggest"]'
        );
        if (items.length > 0) return items.length;
        // Fallback: count .truncate
        return document.querySelectorAll('.truncate').length;
    })()
    "#;
    tab.evaluate(js, false)
        .map(|r| r.value.as_ref().and_then(|v| v.as_f64()).unwrap_or(0.0) as usize)
        .unwrap_or(0)
}

fn list_clickable_items(tab: &headless_chrome::Tab, label: &str) -> usize {
    let js = r#"
    (() => {
        const results = [];

        // Primary: conversation/contact items
        const items = document.querySelectorAll(
            '[class*="conv-item"], [class*="contact-item"], [class*="chat-item"],
             [class*="thread"], [class*="search-result"], [class*="suggest"]'
        );

        if (items.length > 0) {
            items.forEach((item, i) => {
                if (i >= 30) return;
                const name = item.querySelector('.truncate, [class*="name"], [class*="title"]');
                const preview = item.querySelector('[class*="preview"], [class*="last-msg"], [class*="subtitle"]');
                const time = item.querySelector('[class*="time"], [class*="date"]');
                const unread = item.querySelector('[class*="unread"], [class*="badge"]');
                results.push({
                    name: name?.textContent?.trim() || item.textContent?.trim()?.slice(0, 40) || 'Unknown',
                    preview: preview?.textContent?.trim() || '',
                    time: time?.textContent?.trim() || '',
                    unread: unread?.textContent?.trim() || '',
                });
            });
        }

        // Fallback: .truncate elements (clickable parents)
        if (results.length === 0) {
            document.querySelectorAll('.truncate').forEach((el, i) => {
                if (i >= 30) return;
                // Get sibling/parent info
                const parent = el.closest('[class*="conv"], [class*="contact"], [class*="item"]') || el.parentElement;
                const preview = parent?.querySelector('[class*="preview"], [class*="last-msg"], [class*="subtitle"]');
                const time = parent?.querySelector('[class*="time"], [class*="date"]');
                results.push({
                    name: el.textContent?.trim() || 'Unknown',
                    preview: preview?.textContent?.trim() || '',
                    time: time?.textContent?.trim() || '',
                    unread: '',
                });
            });
        }

        return JSON.stringify(results);
    })()
    "#;

    match tab.evaluate(js, false) {
        Ok(result) => {
            let json_str = result.value.as_ref().and_then(|v| v.as_str()).unwrap_or("[]");
            let items: Vec<serde_json::Value> = serde_json::from_str(json_str).unwrap_or_default();

            if items.is_empty() {
                println!("  Không tìm thấy items. Thử [4] xem page info.");
                return 0;
            }

            println!("  {} ({}):\n", label, items.len());
            for (i, item) in items.iter().enumerate() {
                let name = item["name"].as_str().unwrap_or("?");
                let unread = item["unread"].as_str().unwrap_or("");
                let badge = if !unread.is_empty() { format!(" [{}]", unread) } else { String::new() };
                let preview = item["preview"].as_str().unwrap_or("");
                let time = item["time"].as_str().unwrap_or("");
                println!("  {}. {}{}", i + 1, name, badge);
                if !preview.is_empty() || !time.is_empty() {
                    println!("     {} {}", preview, time);
                }
            }
            items.len()
        }
        Err(e) => {
            eprintln!("  ❌ JS error: {}", e);
            0
        }
    }
}

fn click_item_by_index(tab: &headless_chrome::Tab, idx: usize) {
    if idx == 0 {
        println!("  Số phải >= 1");
        return;
    }
    let js_idx = idx - 1; // 0-based

    // Step 1: Get the bounding rect of the element via JS
    let rect_js = format!(r#"
    (() => {{
        let items = document.querySelectorAll(
            '[class*="conv-item"], [class*="contact-item"], [class*="chat-item"],
             [class*="thread"], [class*="search-result"], [class*="suggest"]'
        );

        if (items.length === 0) {{
            const truncates = document.querySelectorAll('.truncate');
            const clickables = [];
            truncates.forEach(el => {{
                const parent = el.closest('[class*="conv"], [class*="contact"], [class*="item"]') || el.parentElement;
                if (parent) clickables.push(parent);
                else clickables.push(el);
            }});
            items = clickables;
        }}

        if ({idx} >= items.length) {{
            return JSON.stringify({{ ok: false, error: "Index out of range: " + items.length + " items" }});
        }}

        const target = items[{idx}];
        // Scroll into view first
        target.scrollIntoView({{ block: 'center' }});

        const rect = target.getBoundingClientRect();
        const name = target.querySelector('.truncate, [class*="name"]')?.textContent?.trim()
                  || target.textContent?.trim()?.slice(0, 30)
                  || 'Unknown';

        return JSON.stringify({{
            ok: true,
            x: Math.round(rect.x + rect.width / 2),
            y: Math.round(rect.y + rect.height / 2),
            name: name
        }});
    }})()
    "#, idx = js_idx);

    match tab.evaluate(&rect_js, false) {
        Ok(r) => {
            let json_str = r.value.as_ref().and_then(|v| v.as_str()).unwrap_or("{}");
            let result: serde_json::Value = serde_json::from_str(json_str).unwrap_or_default();

            if result["ok"].as_bool() != Some(true) {
                println!("  ❌ {}", result["error"].as_str().unwrap_or("Element not found"));
                return;
            }

            let x = result["x"].as_f64().unwrap_or(0.0);
            let y = result["y"].as_f64().unwrap_or(0.0);
            let name = result["name"].as_str().unwrap_or("?");

            println!("  Clicking '{}' at ({}, {})...", name, x, y);

            // Step 2: Use CDP Input.dispatchMouseEvent — real mouse click at coordinates
            // This triggers React/Zalo event handlers properly
            use headless_chrome::protocol::cdp::Input::{
                DispatchMouseEvent, DispatchMouseEventTypeOption,
            };

            let press = tab.call_method(DispatchMouseEvent {
                Type: DispatchMouseEventTypeOption::MousePressed,
                x,
                y,
                button: Some(headless_chrome::protocol::cdp::Input::MouseButton::Left),
                click_count: Some(1),
                buttons: None,
                modifiers: None,
                delta_x: None,
                delta_y: None,
                force: None,
                pointer_Type: None,
                tangential_pressure: None,
                tilt_x: None,
                tilt_y: None,
                timestamp: None,
                twist: None,
            });

            if let Err(e) = press {
                println!("  ❌ MousePressed failed: {}", e);
                return;
            }

            std::thread::sleep(Duration::from_millis(100));

            let release = tab.call_method(DispatchMouseEvent {
                Type: DispatchMouseEventTypeOption::MouseReleased,
                x,
                y,
                button: Some(headless_chrome::protocol::cdp::Input::MouseButton::Left),
                click_count: Some(1),
                buttons: None,
                modifiers: None,
                delta_x: None,
                delta_y: None,
                force: None,
                pointer_Type: None,
                tangential_pressure: None,
                tilt_x: None,
                tilt_y: None,
                timestamp: None,
                twist: None,
            });

            if let Err(e) = release {
                println!("  ❌ MouseReleased failed: {}", e);
                return;
            }

            println!("  ✅ Clicked: {}", name);
            println!("  Đợi load (2s)...");
            std::thread::sleep(Duration::from_secs(2));
        }
        Err(e) => eprintln!("  ❌ {}", e),
    }
}

fn read_messages(tab: &headless_chrome::Tab) {
    println!("\n💬 Đọc tin nhắn conversation đang mở...\n");

    let js = r#"
    (() => {
        const messages = [];

        // Find message elements in chat area
        const chatArea = document.querySelector(
            '[class*="chat-body"], [class*="message-list"], [class*="conversation-body"], #chatArea'
        ) || document.body;

        // Method 1: Look for message containers
        const msgContainers = chatArea.querySelectorAll(
            '[class*="msg-"], [class*="message"], [class*="chat-message"]'
        );

        for (const container of msgContainers) {
            const textEl = container.querySelector('.text, [class*="msg-content"], [class*="text"]');
            const content = textEl?.textContent?.trim();
            if (!content) continue;

            const nameEl = container.querySelector('.truncate, [class*="sender"], [class*="name"]');
            const timeEl = container.querySelector('[class*="time"], [class*="send-time"]');

            messages.push({
                sender: nameEl?.textContent?.trim() || 'Unknown',
                content: content,
                time: timeEl?.textContent?.trim() || '',
            });
        }

        // Method 2: Fallback — find .text elements
        if (messages.length === 0) {
            const textEls = chatArea.querySelectorAll('.text');
            for (const el of textEls) {
                if (el.textContent.trim()) {
                    messages.push({
                        sender: 'Unknown',
                        content: el.textContent.trim(),
                        time: '',
                    });
                }
            }
        }

        return JSON.stringify(messages.slice(-30)); // Last 30 messages
    })()
    "#;

    match tab.evaluate(js, false) {
        Ok(r) => {
            let json_str = r.value.as_ref().and_then(|v| v.as_str()).unwrap_or("[]");
            let messages: Vec<serde_json::Value> = serde_json::from_str(json_str).unwrap_or_default();

            if messages.is_empty() {
                println!("  Không tìm thấy tin nhắn. Hãy click vào 1 conversation trước.");
            } else {
                println!("  {} tin nhắn:\n", messages.len());
                for msg in &messages {
                    let sender = msg["sender"].as_str().unwrap_or("?");
                    let content = msg["content"].as_str().unwrap_or("");
                    let time = msg["time"].as_str().unwrap_or("");
                    if !time.is_empty() {
                        println!("  [{time}] {sender}: {content}");
                    } else {
                        println!("  {sender}: {content}");
                    }
                }
            }
        }
        Err(e) => eprintln!("  ❌ JS error: {}", e),
    }
}

fn page_info(tab: &headless_chrome::Tab) {
    println!("\nℹ️  Thông tin trang hiện tại:\n");

    let js = r#"
    JSON.stringify({
        title: document.title,
        url: window.location.href,
        bodyLen: document.body ? document.body.innerHTML.length : 0,
        elements: document.querySelectorAll('*').length,
        divs: document.querySelectorAll('div').length,
        spans: document.querySelectorAll('span').length,
        imgs: document.querySelectorAll('img').length,
        inputs: document.querySelectorAll('input').length,
        truncates: document.querySelectorAll('.truncate').length,
        texts: document.querySelectorAll('.text').length,
        classes: Array.from(new Set(
            Array.from(document.querySelectorAll('[class]'))
            .flatMap(el => typeof el.className === 'string' ? el.className.split(' ').filter(Boolean) : [])
        )).filter(c => c.includes('chat') || c.includes('msg') || c.includes('conv')
            || c.includes('sidebar') || c.includes('input') || c.includes('search')
            || c.includes('truncate') || c.includes('contact')).slice(0, 30),
    })
    "#;

    match tab.evaluate(js, false) {
        Ok(r) => {
            let json_str = r.value.as_ref().and_then(|v| v.as_str()).unwrap_or("{}");
            let info: serde_json::Value = serde_json::from_str(json_str).unwrap_or_default();

            println!("  Title: {}", info["title"]);
            println!("  URL: {}", info["url"]);
            println!("  Body HTML: {} chars", info["bodyLen"]);
            println!("  Elements: {} total", info["elements"]);
            println!("  Divs: {}, Spans: {}, Imgs: {}", info["divs"], info["spans"], info["imgs"]);
            println!("  Inputs: {}", info["inputs"]);
            println!("  .truncate: {} (contact names)", info["truncates"]);
            println!("  .text: {} (message content)", info["texts"]);
            println!("  Relevant CSS classes: {:?}", info["classes"]);
        }
        Err(e) => eprintln!("  ❌ JS error: {}", e),
    }
}

fn take_screenshot(tab: &headless_chrome::Tab, dir: &PathBuf) {
    let path = dir.join("zalo_web_screenshot.png");
    println!("\n📸 Chụp screenshot...");
    match tab.capture_screenshot(
        headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
        None,
        None,
        true,
    ) {
        Ok(data) => {
            if let Err(e) = std::fs::write(&path, &data) {
                eprintln!("  ❌ Save failed: {}", e);
            } else {
                println!("  ✅ Saved to {}", path.display());
                // Open in Preview
                let _ = std::process::Command::new("open").arg(&path).spawn();
            }
        }
        Err(e) => eprintln!("  ❌ Screenshot failed: {}", e),
    }
}

fn read_line() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or(0);
    input
}
