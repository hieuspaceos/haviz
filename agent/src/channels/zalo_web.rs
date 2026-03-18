use crate::channels::traits::{ChannelReader, ChannelSender};
use crate::message_parser::{self, ParsedMessage};
use headless_chrome::{Browser, LaunchOptions};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

const ZALO_WEB_URL: &str = "https://chat.zalo.me";
const COOKIE_FILE: &str = "zalo_web_cookies.json";

pub struct ZaloWebChannel {
    browser: Mutex<Option<Browser>>,
    chrome_path: Option<PathBuf>,
    cookie_dir: PathBuf,
    my_name: String,
}

impl ZaloWebChannel {
    pub fn new(chrome_path: Option<PathBuf>, cookie_dir: PathBuf, my_name: String) -> Self {
        Self {
            browser: Mutex::new(None),
            chrome_path,
            cookie_dir,
            my_name,
        }
    }

    /// Launch chrome-headless-shell and navigate to Zalo Web.
    /// First time: opens visible window for QR login.
    /// After login: saves cookies, subsequent launches use headless + cookies.
    pub fn ensure_browser(&self) -> Result<(), String> {
        let mut browser_lock = self.browser.lock().unwrap();
        if browser_lock.is_some() {
            return Ok(());
        }

        let has_cookies = self.cookie_dir.join(COOKIE_FILE).exists();

        let mut launch_opts = LaunchOptions::default_builder();
        launch_opts
            .idle_browser_timeout(Duration::from_secs(600))
            .sandbox(false);

        if let Some(ref path) = self.chrome_path {
            launch_opts.path(Some(path.clone()));
        }

        // If no cookies yet, launch visible so user can scan QR
        if !has_cookies {
            launch_opts.headless(false);
            tracing::info!("No Zalo Web cookies found. Opening browser for QR login...");
        } else {
            launch_opts.headless(true);
        }

        let browser = Browser::new(launch_opts.build().map_err(|e| e.to_string())?)
            .map_err(|e| format!("Failed to launch chrome: {}", e))?;

        // Navigate to Zalo Web
        let tab = browser
            .new_tab()
            .map_err(|e| format!("Failed to create tab: {}", e))?;

        // Load cookies if available
        if has_cookies {
            if let Err(e) = self.load_cookies(&tab) {
                tracing::warn!("Failed to load cookies: {}", e);
            }
        }

        tab.navigate_to(ZALO_WEB_URL)
            .map_err(|e| format!("Failed to navigate: {}", e))?;
        tab.wait_until_navigated()
            .map_err(|e| format!("Navigation timeout: {}", e))?;

        // Wait for JS
        std::thread::sleep(Duration::from_secs(3));

        // Check if logged in
        let logged_in = tab
            .evaluate("!!document.querySelector('.truncate')", false)
            .map(|r| r.value.as_ref().and_then(|v| v.as_bool()).unwrap_or(false))
            .unwrap_or(false);

        if logged_in {
            tracing::info!("Zalo Web: logged in!");
            if let Err(e) = self.save_cookies(&tab) {
                tracing::warn!("Failed to save cookies: {}", e);
            }
        } else if !has_cookies {
            tracing::info!("Waiting for QR scan... (scan QR code in the browser window)");
            // Wait up to 120 seconds for user to scan QR
            for i in 0..40 {
                std::thread::sleep(Duration::from_secs(3));
                let check = tab
                    .evaluate("!!document.querySelector('.truncate')", false)
                    .map(|r| r.value.as_ref().and_then(|v| v.as_bool()).unwrap_or(false))
                    .unwrap_or(false);
                if check {
                    tracing::info!("QR scan detected! Saving cookies...");
                    if let Err(e) = self.save_cookies(&tab) {
                        tracing::warn!("Failed to save cookies: {}", e);
                    }
                    break;
                }
                if i % 10 == 0 && i > 0 {
                    tracing::info!("Still waiting for QR scan... ({}s)", i * 3);
                }
            }
        }

        *browser_lock = Some(browser);
        Ok(())
    }

    fn save_cookies(
        &self,
        tab: &headless_chrome::Tab,
    ) -> Result<(), String> {
        let cookies = tab
            .get_cookies()
            .map_err(|e| format!("Get cookies failed: {}", e))?;

        let json = serde_json::to_string_pretty(&cookies)
            .map_err(|e| format!("Serialize cookies failed: {}", e))?;

        std::fs::create_dir_all(&self.cookie_dir).ok();
        std::fs::write(self.cookie_dir.join(COOKIE_FILE), json)
            .map_err(|e| format!("Write cookies failed: {}", e))?;

        tracing::info!(
            "Saved {} cookies to {}",
            cookies.len(),
            self.cookie_dir.join(COOKIE_FILE).display()
        );
        Ok(())
    }

    fn load_cookies(
        &self,
        tab: &headless_chrome::Tab,
    ) -> Result<(), String> {
        let path = self.cookie_dir.join(COOKIE_FILE);
        let json =
            std::fs::read_to_string(&path).map_err(|e| format!("Read cookies failed: {}", e))?;

        let cookies: Vec<headless_chrome::protocol::cdp::Network::CookieParam> =
            serde_json::from_str(&json).map_err(|e| format!("Parse cookies failed: {}", e))?;

        for cookie in &cookies {
            tab.set_cookies(vec![cookie.clone()])
                .map_err(|e| format!("Set cookie failed: {}", e))?;
        }

        tracing::info!("Loaded {} cookies from {}", cookies.len(), path.display());
        Ok(())
    }

    /// Extract messages from the current Zalo Web page via JS evaluation
    fn extract_messages_js(
        &self,
        tab: &headless_chrome::Tab,
    ) -> Result<Vec<ParsedMessage>, String> {
        let js = r#"
        (() => {
            const messages = [];
            // Zalo Web DOM: .text = message content, .truncate = sender name
            // .card-send-time__sendTime = timestamp
            const msgEls = document.querySelectorAll('[class*="message"], [class*="msg-"]');

            // Fallback: scan all text nodes in chat area
            const chatArea = document.querySelector('[class*="chat-body"], [class*="conversation"], #chatArea')
                          || document.body;

            const textEls = chatArea.querySelectorAll('.text, [class*="msg-content"]');
            const nameEls = chatArea.querySelectorAll('.truncate, [class*="sender"]');
            const timeEls = chatArea.querySelectorAll('.card-send-time__sendTime, [class*="time"]');

            // Simple extraction: pair texts with nearest name/time
            for (let i = 0; i < textEls.length; i++) {
                const content = textEls[i]?.textContent?.trim();
                if (!content) continue;

                // Find nearest sender name (look upward in DOM)
                let sender = 'Unknown';
                let el = textEls[i];
                for (let j = 0; j < 10; j++) {
                    el = el?.parentElement;
                    if (!el) break;
                    const nameEl = el.querySelector('.truncate, [class*="sender"]');
                    if (nameEl?.textContent?.trim()) {
                        sender = nameEl.textContent.trim();
                        break;
                    }
                }

                // Find nearest timestamp
                let timestamp = '';
                el = textEls[i];
                for (let j = 0; j < 10; j++) {
                    el = el?.parentElement;
                    if (!el) break;
                    const timeEl = el.querySelector('.card-send-time__sendTime, [class*="time"]');
                    if (timeEl?.textContent?.trim()) {
                        timestamp = timeEl.textContent.trim();
                        break;
                    }
                }

                messages.push({ sender, content, timestamp: timestamp || 'unknown' });
            }

            return JSON.stringify(messages);
        })()
        "#;

        let result = tab
            .evaluate(js, false)
            .map_err(|e| format!("JS evaluate failed: {}", e))?;

        let json_str = result
            .value
            .as_ref()
            .and_then(|v| v.as_str())
            .unwrap_or("[]");

        let raw_msgs: Vec<message_parser::RawMessage> =
            serde_json::from_str(json_str).unwrap_or_default();

        let mut messages = Vec::new();
        for raw in raw_msgs {
            let direction = message_parser::determine_direction(&raw.sender, &self.my_name);
            let hash = message_parser::compute_hash(&raw.sender, &raw.content, &raw.timestamp);
            messages.push(ParsedMessage {
                sender: raw.sender,
                content: raw.content,
                timestamp: raw.timestamp,
                direction,
                content_hash: hash,
            });
        }

        Ok(messages)
    }
}

impl ChannelReader for ZaloWebChannel {
    fn read_messages(&self) -> Result<Vec<ParsedMessage>, String> {
        self.ensure_browser()?;

        let browser_lock = self.browser.lock().unwrap();
        let browser = browser_lock
            .as_ref()
            .ok_or("Browser not initialized")?;

        let tabs = browser.get_tabs().lock().unwrap();
        let tab = tabs.first().ok_or("No tab available")?;

        self.extract_messages_js(tab)
    }
}

impl ChannelSender for ZaloWebChannel {
    fn send_message(&self, to: &str, message: &str) -> Result<(), String> {
        self.ensure_browser()?;

        let browser_lock = self.browser.lock().unwrap();
        let browser = browser_lock
            .as_ref()
            .ok_or("Browser not initialized")?;

        let tabs = browser.get_tabs().lock().unwrap();
        let tab = tabs.first().ok_or("No tab available")?;

        // Search for contact
        let search_js = format!(
            r#"
            (() => {{
                const searchInput = document.querySelector('input[placeholder*="Tìm"], input[type="text"]');
                if (searchInput) {{
                    searchInput.focus();
                    searchInput.value = '{}';
                    searchInput.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    return true;
                }}
                return false;
            }})()
            "#,
            to.replace('\'', "\\'").replace('"', "\\\"")
        );

        tab.evaluate(&search_js, false)
            .map_err(|e| format!("Search failed: {}", e))?;

        std::thread::sleep(Duration::from_millis(1500));

        // Click first search result
        tab.evaluate(
            r#"
            (() => {
                const result = document.querySelector('[class*="conv-item"], [class*="search-result"]');
                if (result) { result.click(); return true; }
                return false;
            })()
            "#,
            false,
        )
        .map_err(|e| format!("Click result failed: {}", e))?;

        std::thread::sleep(Duration::from_millis(1000));

        // Type message and send
        let send_js = format!(
            r#"
            (() => {{
                const input = document.querySelector('[contenteditable="true"], textarea[class*="input"], #chatInput');
                if (input) {{
                    input.focus();
                    input.textContent = '{}';
                    input.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    // Press Enter
                    input.dispatchEvent(new KeyboardEvent('keydown', {{ key: 'Enter', keyCode: 13, bubbles: true }}));
                    return true;
                }}
                return false;
            }})()
            "#,
            message.replace('\'', "\\'").replace('"', "\\\"").replace('\n', "\\n")
        );

        tab.evaluate(&send_js, false)
            .map_err(|e| format!("Send failed: {}", e))?;

        Ok(())
    }
}
