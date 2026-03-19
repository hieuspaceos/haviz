/// Inline JavaScript snippets injected into the Zalo WebView.
///
/// Each constant is a self-executing function that interacts with
/// the Zalo Web DOM. Extracted from zalo_control.rs to keep
/// handler files under 200 LOC.

/// Extracts conversation list from Zalo sidebar truncate elements.
/// Posts result via `window.ipc.postMessage({ type: 'conversations', data })`.
pub const JS_EXTRACT_CONVERSATIONS: &str = r#"(function(){
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
})();"#;

/// Extracts visible chat messages from the currently open conversation.
/// Posts deduplicated result (last 20) via `window.ipc.postMessage`.
pub const JS_EXTRACT_MESSAGES: &str = r#"(function(){
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
})();"#;

/// Clears the search input field.
pub const JS_CLEAR_INPUT: &str = r#"(function(){
    var inp=document.querySelector('input[type="text"]');
    if(!inp)inp=document.querySelector('input');
    if(!inp)return;
    inp.focus();
    inp.value='';
    inp.dispatchEvent(new Event('input',{bubbles:true}));
})();"#;

/// Presses Enter in the search input to open the first result.
pub const JS_ENTER_SEARCH: &str = r#"(function(){
    var inp=document.querySelector('input[type="text"]');
    if(!inp)inp=document.querySelector('input');
    if(!inp)return;
    inp.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',code:'Enter',keyCode:13,which:13,bubbles:true}));
    inp.dispatchEvent(new KeyboardEvent('keypress',{key:'Enter',code:'Enter',keyCode:13,which:13,bubbles:true}));
    inp.dispatchEvent(new KeyboardEvent('keyup',{key:'Enter',code:'Enter',keyCode:13,which:13,bubbles:true}));
})();"#;

/// Focuses the contenteditable chat input (JS-only, for Windows).
pub const JS_FOCUS_CHAT_INPUT: &str = r#"(function(){
    var el=document.querySelector('[contenteditable="true"]');
    if(el){el.focus();el.click();}
})();"#;

/// Presses Enter and clicks any visible send button in the chat area.
pub const JS_SEND_ENTER: &str = r#"(function(){
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
})();"#;

/// Generates JS to type text char-by-char into a search input using nativeInputValueSetter.
pub fn js_type_search(query: &str) -> String {
    let escaped = query.replace('\\', "\\\\").replace('"', "\\\"").replace('\'', "\\'");
    format!(
        r#"(function(){{
            var inp=document.querySelector('input[type="text"]');
            if(!inp)inp=document.querySelector('input');
            if(!inp)return;
            inp.focus();
            var setter=Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype,'value').set;
            var text='{}';
            for(var i=0;i<text.length;i++){{
                var ch=text[i];
                inp.dispatchEvent(new KeyboardEvent('keydown',{{key:ch,code:'Key'+ch.toUpperCase(),bubbles:true}}));
                setter.call(inp,text.substring(0,i+1));
                inp.dispatchEvent(new InputEvent('input',{{bubbles:true,data:ch,inputType:'insertText'}}));
                inp.dispatchEvent(new KeyboardEvent('keyup',{{key:ch,code:'Key'+ch.toUpperCase(),bubbles:true}}));
            }}
        }})();"#,
        escaped
    )
}

/// Generates JS to type a message into the contenteditable chat input.
pub fn js_type_message(message: &str) -> String {
    let escaped = message.replace('\\', "\\\\").replace('"', "\\\"");
    format!(
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
        escaped
    )
}

/// Generates JS to click the Nth item in the conversation list (0-based).
pub fn js_click_conversation(idx: usize) -> String {
    format!(
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
            }}
        }})();"#,
        idx = idx
    )
}

/// Generates JS to clear input then type search query using nativeInputValueSetter.
pub fn js_clear_and_type_search(query: &str) -> String {
    let escaped = query.replace('\\', "\\\\").replace('"', "\\\"").replace('\'', "\\'");
    format!(
        r#"(function(){{
            var inp=document.querySelector('input[type="text"]');
            if(!inp)inp=document.querySelector('input');
            if(!inp)return;
            inp.focus();
            var setter=Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype,'value').set;
            setter.call(inp,'');
            inp.dispatchEvent(new InputEvent('input',{{bubbles:true,inputType:'deleteContentBackward'}}));
            var text='{}';
            for(var i=0;i<text.length;i++){{
                var ch=text[i];
                inp.dispatchEvent(new KeyboardEvent('keydown',{{key:ch,bubbles:true}}));
                setter.call(inp,text.substring(0,i+1));
                inp.dispatchEvent(new InputEvent('input',{{bubbles:true,data:ch,inputType:'insertText'}}));
                inp.dispatchEvent(new KeyboardEvent('keyup',{{key:ch,bubbles:true}}));
            }}
        }})();"#,
        escaped
    )
}
