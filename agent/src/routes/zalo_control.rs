/// Route handlers for Zalo Web sidebar control.
///
/// All handlers interact with the Zalo WebView via eval_zalo_js() (IPC queue) or
/// via AppleScript run_osascript() for OS-level mouse/keyboard actions.
use axum::{extract::Query, response::Json, response::Response};
use serde::Deserialize;

use crate::app::ipc::{eval_zalo_js, ZALO_CONVERSATIONS, ZALO_MESSAGES};
#[cfg(target_os = "macos")]
use crate::platform::macos::osascript::run_osascript;

// ── Request structs ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SearchRequest {
    pub query: String,
}

#[derive(Deserialize)]
pub struct OpenRequest {
    pub index: usize,
}

#[derive(Deserialize)]
pub struct SendMsgRequest {
    pub message: String,
}

#[derive(Deserialize)]
pub struct SearchAndSendRequest {
    pub to: String,
    pub message: String,
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Escape backslash and double-quote for embedding in JS string literals.
fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// GET /api/zalo/conversations
/// Injects JS into the Zalo sidebar to extract the conversation list, waits for
/// the IPC callback, and returns the result.
pub async fn zalo_conversations_handler() -> Json<serde_json::Value> {
    *ZALO_CONVERSATIONS.lock().unwrap() = None;

    let _ = eval_zalo_js(
        r#"(function(){
        var convs=[];
        var seen=new Set();
        var skip=new Set(['Tin nhắn','Danh bạ','Zalo Cloud','My Documents','Công cụ',
            'Cài đặt','Tìm kiếm','Tất cả','Chưa đọc','Phân loại','Đóng','Tải ngay']);
        var truncates=document.querySelectorAll('.truncate');
        truncates.forEach(function(el){
            var name=el.textContent?el.textContent.trim():'';
            if(!name||name.length>50||name.length<1||seen.has(name)||skip.has(name))return;
            if(name.indexOf('Bạn:')===0)return;
            var cls=(typeof el.className==='string')?el.className:'';
            if(cls.indexOf('lb-tab-title')>=0)return;
            if(cls.indexOf('conv-dbname')>=0||cls.indexOf('subtitle')>=0)return;
            var isConv=false;
            var ancestor=el.parentElement;
            for(var i=0;i<8&&ancestor;i++){
                var aCls=(typeof ancestor.className==='string')?ancestor.className:'';
                if(aCls.indexOf('conv')>=0||aCls.indexOf('contact')>=0||aCls.indexOf('chat-item')>=0){
                    isConv=true;break;
                }
                ancestor=ancestor.parentElement;
            }
            if(!isConv)return;
            var parent=el.parentElement;
            if(parent){
                var siblings=parent.querySelectorAll('.truncate');
                if(siblings.length>1&&siblings[0]!==el)return;
            }
            seen.add(name);
            convs.push({name:name,time:'',preview:'',sender:''});
        });
        if(window.ipc&&window.ipc.postMessage){
            window.ipc.postMessage(JSON.stringify({type:'conversations',data:convs}));
        }
    })();"#,
    );

    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if ZALO_CONVERSATIONS.lock().unwrap().is_some() {
            break;
        }
    }

    let data = ZALO_CONVERSATIONS.lock().unwrap().take();
    match data {
        Some(convs) => Json(serde_json::json!({ "ok": true, "conversations": convs })),
        None => Json(serde_json::json!({ "ok": true, "conversations": [] })),
    }
}

/// GET /api/zalo/messages
/// Injects JS to extract visible chat messages from the open conversation,
/// waits for the IPC callback, and returns the deduplicated list.
pub async fn zalo_messages_handler() -> Json<serde_json::Value> {
    *ZALO_MESSAGES.lock().unwrap() = None;

    let _ = eval_zalo_js(r#"(function(){
        var messages=[];
        var skip=new Set([
            'Tin nhắn','Danh bạ','Zalo Cloud','My Documents','Công cụ','Cài đặt',
            'Tìm kiếm','Tất cả','Chưa đọc','Phân loại','Đóng','Tải ngay',
            'Gửi nhanh','Đồng bộ ngay','Hôm nay','Hôm qua','Đã gửi','Đã xem',
            'Xem trước khi gửi','Thả File hoặc Ảnh vào đây để gửi nhanh',
            'Thả File hoặc Ảnh vào đây để xem lại trước khi gửi',
            'phút','giờ','ngày','thành viên','Nodaking',
            'Sử dụng Zalo PC để lưu trữ dài hạn và dễ dàng tìm kiếm đầy đủ dữ liệu trò chuyện của bạn.',
            'Hình ảnh','Video','File','Link','Bình chọn',
        ]);
        var emojiRe=/^[\/:][a-z-]+$|^[\/:][a-z][\w-]*$/;
        var timeRe=/^\d{1,2}:\d{2}$/;
        var shortRe=/^\d+$/;
        var memberRe=/^\d+\s*thành viên$/;
        var validTags=new Set(['SPAN','DIV','P','A','EM','STRONG','B','I']);
        var all=document.querySelectorAll('*');
        for(var i=0;i<all.length;i++){
            var el=all[i];
            if(el.children.length>0)continue;
            if(!validTags.has(el.tagName))continue;
            var text=el.textContent?el.textContent.trim():'';
            if(text.length<2||text.length>500)continue;
            if(skip.has(text))continue;
            if(emojiRe.test(text))continue;
            if(timeRe.test(text))continue;
            if(shortRe.test(text))continue;
            if(memberRe.test(text))continue;
            if(text.length<=2)continue;
            if(text.length<=3&&/^[:;]/.test(text))continue;
            var cls=(typeof el.className==='string')?el.className:'';
            if(cls.indexOf('lb-tab')>=0||cls.indexOf('banner')>=0||cls.indexOf('fake-text')>=0)continue;
            var content=text;
            if(cls.indexOf('conv-dbname')>=0||text.endsWith(':'))continue;
            content=content.replace(/\/?-strong/g,'').replace(/\/?-heart/g,'')
                .replace(/:>/g,'').replace(/:o/g,'').replace(/:-\(\(/g,'').replace(/:-h/g,'')
                .replace(/\d{1,2}:\d{2}/g,'').replace(/Đã gửi/g,'').replace(/Đã xem/g,'')
                .trim();
            if(!content||content.length<2)continue;
            messages.push({sender:'',content:content,class:cls.substring(0,40)});
        }
        var seen=new Set();
        var unique=[];
        for(var j=messages.length-1;j>=0;j--){
            var key=messages[j].content;
            if(!seen.has(key)){seen.add(key);unique.unshift(messages[j]);}
        }
        if(window.ipc&&window.ipc.postMessage){
            window.ipc.postMessage(JSON.stringify(unique.slice(-20)));
        }
    })();"#);

    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if ZALO_MESSAGES.lock().unwrap().is_some() {
            break;
        }
    }

    let data = ZALO_MESSAGES.lock().unwrap().take();
    match data {
        Some(msgs) => Json(serde_json::json!({ "ok": true, "messages": msgs })),
        None => Json(serde_json::json!({
            "ok": false,
            "messages": [],
            "note": "No messages extracted. Open a conversation in Zalo sidebar first.",
        })),
    }
}

/// Internal GET /api/zalo/_messages_callback
/// Receives messages via query param (image-src trick from injected JS).
pub async fn zalo_messages_callback(
    query: Query<std::collections::HashMap<String, String>>,
) -> Response {
    if let Some(data) = query.get("data") {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
            *ZALO_MESSAGES.lock().unwrap() = Some(parsed);
        }
    }
    // Return 1×1 transparent PNG so JS image.src trick gets a valid response
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

/// POST /api/zalo/search
/// Types a search query into the Zalo sidebar's search input and presses Enter
/// to open the first result.
pub async fn zalo_search_handler(
    axum::extract::Json(req): axum::extract::Json<SearchRequest>,
) -> Json<serde_json::Value> {
    let query = req.query.clone();
    let auto_enter = !query.is_empty();

    // Clear input
    let js_type = r#"(function(){
        var inp=document.querySelector('input[type="text"]');
        if(!inp)inp=document.querySelector('input');
        if(!inp)return;
        inp.focus();
        inp.value='';
        inp.dispatchEvent(new Event('input',{bubbles:true}));
    })();"#;
    let _ = eval_zalo_js(js_type);
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Type query char by char using native input setter (bypasses React synthetic events)
    let js_set = format!(
        r#"(function(){{
            var inp=document.querySelector('input[type="text"]');
            if(!inp)inp=document.querySelector('input');
            if(!inp)return;
            inp.focus();
            inp.value='';
            var nativeInputValueSetter=Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype,'value').set;
            var text='{}';
            for(var i=0;i<text.length;i++){{
                var char=text[i];
                inp.dispatchEvent(new KeyboardEvent('keydown',{{key:char,code:'Key'+char.toUpperCase(),bubbles:true}}));
                nativeInputValueSetter.call(inp,text.substring(0,i+1));
                inp.dispatchEvent(new InputEvent('input',{{bubbles:true,data:char,inputType:'insertText'}}));
                inp.dispatchEvent(new KeyboardEvent('keyup',{{key:char,code:'Key'+char.toUpperCase(),bubbles:true}}));
            }}
        }})();"#,
        esc(&query).replace('\'', "\\'")
    );
    let _ = eval_zalo_js(&js_set);
    std::thread::sleep(std::time::Duration::from_millis(2500));

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

    Json(serde_json::json!({ "ok": true, "query": req.query, "auto_enter": auto_enter }))
}

/// POST /api/zalo/open
/// Clicks the Nth conversation item in the sidebar list (1-based index).
pub async fn zalo_open_handler(
    axum::extract::Json(req): axum::extract::Json<OpenRequest>,
) -> Json<serde_json::Value> {
    let idx = req.index.saturating_sub(1);
    let js = format!(
        r#"(function(){{
            var items=document.querySelectorAll('[class*="conv-item"],[class*="contact-item"],[class*="chat-item"]');
            if(items.length===0){{
                var all=document.querySelectorAll('*');
                var clickable=[];
                for(var i=0;i<all.length;i++){{
                    var el=all[i];
                    if(el.children.length>0&&el.children.length<10&&el.offsetHeight>40&&el.offsetHeight<100){{
                        var hasText=el.querySelector('span,div');
                        if(hasText&&hasText.textContent.trim().length>1)clickable.push(el);
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
    Json(serde_json::json!({ "ok": true, "clicked_index": req.index }))
}

/// POST /api/zalo/send
/// Types a message into the currently open Zalo conversation and sends it.
/// Uses AppleScript for OS-level click to focus the chat input, then JS InputEvents.
pub async fn zalo_send_handler(
    axum::extract::Json(req): axum::extract::Json<SendMsgRequest>,
) -> Json<serde_json::Value> {
    let message = esc(&req.message);
    let msg = message.clone();

    tokio::task::spawn_blocking(move || {
        // macOS: OS-level click to move keyboard focus into the WebView chat input
        #[cfg(target_os = "macos")]
        {
            let _ = run_osascript("tell application \"System Events\" to click at {1, 1}");

            let pos_result = run_osascript(
                "tell application \"System Events\" to tell process \"haviz_app\" \
                 to return {position of window 1, size of window 1}",
            );

            match pos_result {
                Ok(pos_str) => {
                    let nums: Vec<f64> = pos_str
                        .replace('{', "")
                        .replace('}', "")
                        .split(',')
                        .filter_map(|s| s.trim().parse().ok())
                        .collect();
                    if nums.len() >= 4 {
                        let (wx, wy, ww, wh) = (nums[0], nums[1], nums[2], nums[3]);
                        let click_x = (wx + ww - 200.0) as i64;
                        let click_y = (wy + wh - 60.0) as i64;
                        let _ = run_osascript(&format!(
                            "tell application \"System Events\" to click at {{{}, {}}}",
                            click_x, click_y
                        ));
                    }
                }
                Err(_) => {
                    // Fallback: JS focus
                    eval_zalo_js(
                        r#"(function(){
                        var el=document.querySelector('[contenteditable="true"]');
                        if(el){el.focus();el.click();}
                    })();"#,
                    )
                    .ok();
                }
            }
        }

        // Windows: JS focus is sufficient — WebView handles input natively
        #[cfg(target_os = "windows")]
        {
            eval_zalo_js(
                r#"(function(){
                var el=document.querySelector('[contenteditable="true"]');
                if(el){el.focus();el.click();}
            })();"#,
            )
            .ok();
        }

        std::thread::sleep(std::time::Duration::from_millis(500));

        // Type message char by char via InputEvents
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

        // Enter key + send button click
        eval_zalo_js(r#"(function(){
            var el=document.querySelector('[contenteditable="true"]');
            if(!el)return;
            el.focus();
            ['keydown','keypress','keyup'].forEach(function(type){
                el.dispatchEvent(new KeyboardEvent(type,{
                    key:'Enter',code:'Enter',keyCode:13,which:13,bubbles:true,cancelable:true
                }));
            });
            var btns=document.querySelectorAll('[class*="send"],[class*="Send"],button,[role="button"]');
            for(var i=0;i<btns.length;i++){
                var b=btns[i];
                var r=b.getBoundingClientRect();
                if(r.width>10&&r.width<80&&r.height>10&&r.height<80&&r.bottom>window.innerHeight-100){
                    b.click();break;
                }
            }
        })();"#).ok();
    })
    .await
    .ok();

    Json(serde_json::json!({ "ok": true, "message": req.message }))
}

/// POST /api/zalo/search-and-send
/// Full flow: search contact → open conversation → type message → send.
pub async fn zalo_search_and_send_handler(
    axum::extract::Json(req): axum::extract::Json<SearchAndSendRequest>,
) -> Json<serde_json::Value> {
    let to = esc(&req.to).replace('\'', "\\'");
    let message = esc(&req.message);

    let result = tokio::task::spawn_blocking(move || {
        // Clear search input
        eval_zalo_js(r#"(function(){
            var inp=document.querySelector('input[type="text"]');
            if(!inp)inp=document.querySelector('input');
            if(!inp)return;
            inp.focus();
            var setter=Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype,'value').set;
            setter.call(inp,'');
            inp.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'deleteContentBackward'}));
        })();"#).ok();
        std::thread::sleep(std::time::Duration::from_millis(300));

        // Type contact name char by char
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
        std::thread::sleep(std::time::Duration::from_millis(2500));

        // Enter to open first search result
        eval_zalo_js(r#"(function(){
            var inp=document.querySelector('input[type="text"]');
            if(!inp)inp=document.querySelector('input');
            if(!inp)return;
            inp.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',code:'Enter',keyCode:13,which:13,bubbles:true}));
            inp.dispatchEvent(new KeyboardEvent('keypress',{key:'Enter',code:'Enter',keyCode:13,which:13,bubbles:true}));
            inp.dispatchEvent(new KeyboardEvent('keyup',{key:'Enter',code:'Enter',keyCode:13,which:13,bubbles:true}));
        })();"#).ok();
        std::thread::sleep(std::time::Duration::from_millis(1500));

        // Type message into contenteditable chat input
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

        // Enter + send button click
        eval_zalo_js(r#"(function(){
            var el=document.querySelector('[contenteditable="true"]');
            if(!el)return;
            ['keydown','keypress','keyup'].forEach(function(type){
                el.dispatchEvent(new KeyboardEvent(type,{
                    key:'Enter',code:'Enter',keyCode:13,which:13,bubbles:true,cancelable:true
                }));
            });
            var btns=document.querySelectorAll('[class*="send"],[class*="Send"],button,[role="button"]');
            for(var i=0;i<btns.length;i++){
                var b=btns[i];
                var r=b.getBoundingClientRect();
                if(r.width>10&&r.width<80&&r.height>10&&r.height<80&&r.bottom>window.innerHeight-100){
                    b.click();break;
                }
            }
        })();"#).ok();

        Ok::<_, String>("sent".to_string())
    })
    .await
    .unwrap_or(Err("spawn failed".to_string()));

    Json(serde_json::json!({
        "ok": result.is_ok(),
        "to": req.to,
        "message": req.message,
    }))
}
