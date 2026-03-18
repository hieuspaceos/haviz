// Haviz Extension — Đọc DOM + Gửi qua Agent (localhost:9999)

(function() {
  var AGENT = 'http://localhost:9999';

  var panel = document.createElement('div');
  panel.id = 'haviz-panel';
  panel.innerHTML = `
    <div style="font-weight:bold;font-size:14px;margin-bottom:4px;">HAVIZ</div>
    <div id="haviz-status" style="font-size:10px;margin-bottom:8px;color:#aaa;">⏳ Kiểm tra Agent...</div>
    <button id="haviz-read" style="width:100%;padding:6px;margin-bottom:6px;border:none;border-radius:4px;background:#4CAF50;color:white;cursor:pointer;font-size:12px;">Đọc tin nhắn</button>
    <textarea id="haviz-msg" placeholder="Nhập tin nhắn..." style="width:100%;height:50px;padding:4px;border:1px solid #555;border-radius:4px;background:#333;color:white;font-size:12px;resize:vertical;box-sizing:border-box;"></textarea>
    <button id="haviz-send" style="width:100%;padding:6px;margin-top:6px;border:none;border-radius:4px;background:#2196F3;color:white;cursor:pointer;font-size:12px;">Gửi tin nhắn</button>
    <div id="haviz-result" style="margin-top:8px;font-size:11px;max-height:200px;overflow-y:auto;white-space:pre-wrap;word-break:break-word;"></div>
  `;
  panel.style.cssText = 'position:fixed;top:80px;right:20px;width:250px;background:#1a1a1a;color:#eee;padding:12px;border-radius:8px;z-index:999999;font-family:sans-serif;box-shadow:0 4px 12px rgba(0,0,0,0.5);';
  document.body.appendChild(panel);

  var result = document.getElementById('haviz-result');
  var status = document.getElementById('haviz-status');

  // Kiểm tra Agent
  fetch(AGENT + '/status').then(function(r) { return r.json(); }).then(function(d) {
    status.textContent = '✅ Agent connected';
    status.style.color = '#4CAF50';
  }).catch(function() {
    status.textContent = '❌ Agent offline — chạy: node agent.js';
    status.style.color = '#f44336';
  });

  // === ĐỌC TIN NHẮN ===
  document.getElementById('haviz-read').addEventListener('click', function() {
    var texts = [];
    document.querySelectorAll('div, span, p').forEach(function(el) {
      if (el.children.length === 0 && !panel.contains(el)) {
        var t = (el.innerText || '').trim();
        if (t.length > 1 && t.length < 1000) {
          var skip = ['Tìm kiếm','Tất cả','Chưa đọc','Phân loại','Tin nhắn','Danh bạ','HAVIZ'].indexOf(t) >= 0;
          if (!skip) texts.push(t);
        }
      }
    });
    result.textContent = '--- ' + texts.length + ' elements ---\n\n' + texts.slice(-15).join('\n\n');
    result.scrollTop = result.scrollHeight;
  });

  // === GỬI TIN NHẮN qua Agent ===
  document.getElementById('haviz-send').addEventListener('click', function() {
    var msg = document.getElementById('haviz-msg').value.trim();
    if (!msg) { result.textContent = '⚠ Nhập tin nhắn trước'; return; }

    // Trước khi gửi, click vào ô nhập Zalo để focus
    var input = null;
    var editables = document.querySelectorAll('[contenteditable="true"]');
    for (var i = 0; i < editables.length; i++) {
      if (!panel.contains(editables[i]) && editables[i].offsetHeight > 0) {
        input = editables[i];
        break;
      }
    }
    if (input) {
      input.click();
      input.focus();
    }

    result.textContent = '⏳ Đang gửi qua Agent...';

    // Gửi qua Agent — Agent sẽ paste + Enter
    fetch(AGENT + '/send', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ message: msg, app: 'Google Chrome' })
    }).then(function(r) { return r.json(); }).then(function(d) {
      if (d.ok) {
        result.textContent = '✅ Đã gửi: "' + msg + '"';
        document.getElementById('haviz-msg').value = '';
      } else {
        result.textContent = '❌ Lỗi: ' + d.error;
      }
    }).catch(function(e) {
      result.textContent = '❌ Agent offline.\nChạy: node agent.js';
    });
  });

  // Phím Enter trong textarea cũng gửi
  document.getElementById('haviz-msg').addEventListener('keydown', function(e) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      document.getElementById('haviz-send').click();
    }
  });
})();
