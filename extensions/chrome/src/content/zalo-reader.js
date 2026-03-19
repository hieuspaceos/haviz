// Haviz Extension — Doc DOM + Gui qua Agent (localhost:9999)
// Xac thuc bang Bearer token luu trong chrome.storage.local

(function () {
  var AGENT = 'http://localhost:9999';
  var TOKEN_KEY = 'haviz_agent_token';

  // ---- Build panel ----
  var panel = document.createElement('div');
  panel.id = 'haviz-panel';
  panel.style.cssText =
    'position:fixed;top:80px;right:20px;width:260px;background:#1a1a1a;color:#eee;' +
    'padding:12px;border-radius:8px;z-index:999999;font-family:sans-serif;' +
    'box-shadow:0 4px 12px rgba(0,0,0,0.5);';

  panel.innerHTML =
    '<div style="font-weight:bold;font-size:14px;margin-bottom:4px;">HAVIZ</div>' +
    '<div id="hv-status" style="font-size:10px;margin-bottom:6px;color:#aaa;">Checking agent...</div>' +
    '<button id="hv-read" style="width:100%;padding:6px;margin-bottom:6px;border:none;border-radius:4px;background:#4CAF50;color:white;cursor:pointer;font-size:12px;">Doc tin nhan</button>' +
    '<textarea id="hv-msg" placeholder="Nhap tin nhan..." style="width:100%;height:50px;padding:4px;border:1px solid #555;border-radius:4px;background:#333;color:white;font-size:12px;resize:vertical;box-sizing:border-box;"></textarea>' +
    '<button id="hv-send" style="width:100%;padding:6px;margin-top:6px;border:none;border-radius:4px;background:#2196F3;color:white;cursor:pointer;font-size:12px;">Gui tin nhan</button>' +
    '<hr style="border-color:#333;margin:8px 0;">' +
    '<div style="font-size:10px;color:#aaa;margin-bottom:4px;">Agent token</div>' +
    '<input id="hv-token-input" type="password" placeholder="Bearer token..." style="width:100%;padding:4px;border:1px solid #555;border-radius:4px;background:#333;color:white;font-size:11px;box-sizing:border-box;">' +
    '<button id="hv-token-save" style="width:100%;padding:4px;margin-top:4px;border:none;border-radius:4px;background:#555;color:white;cursor:pointer;font-size:11px;">Save token</button>' +
    '<div id="hv-result" style="margin-top:8px;font-size:11px;max-height:180px;overflow-y:auto;white-space:pre-wrap;word-break:break-word;"></div>';

  document.body.appendChild(panel);

  var result = document.getElementById('hv-result');
  var status = document.getElementById('hv-status');
  var tokenInput = document.getElementById('hv-token-input');

  // ---- Token helpers ----
  function loadToken(cb) {
    chrome.storage.local.get(TOKEN_KEY, function (items) {
      cb(items[TOKEN_KEY] || '');
    });
  }

  function saveToken(token) {
    chrome.storage.local.set({ [TOKEN_KEY]: token });
  }

  function authHeaders(extra) {
    var h = Object.assign({ 'Content-Type': 'application/json' }, extra || {});
    if (tokenInput.value) h['Authorization'] = 'Bearer ' + tokenInput.value;
    return h;
  }

  // Populate token input from storage on load
  loadToken(function (t) {
    tokenInput.value = t;
    checkAgent();
  });

  // Save token button
  document.getElementById('hv-token-save').addEventListener('click', function () {
    var t = tokenInput.value.trim();
    saveToken(t);
    result.textContent = t ? 'Token saved.' : 'Token cleared.';
  });

  // ---- Check agent connectivity ----
  function checkAgent() {
    fetch(AGENT + '/api/status')
      .then(function (r) { return r.json(); })
      .then(function () {
        var hasToken = !!tokenInput.value;
        status.textContent = hasToken ? 'Agent connected' : 'Agent connected — no token set';
        status.style.color = hasToken ? '#4CAF50' : '#FF9800';
      })
      .catch(function () {
        status.textContent = 'Agent offline';
        status.style.color = '#f44336';
      });
  }

  // ---- Read messages from DOM ----
  document.getElementById('hv-read').addEventListener('click', function () {
    var texts = [];
    document.querySelectorAll('div, span, p').forEach(function (el) {
      if (el.children.length === 0 && !panel.contains(el)) {
        var t = (el.innerText || '').trim();
        if (t.length > 1 && t.length < 1000) {
          var skip =
            ['Tim kiem', 'Tat ca', 'Chua doc', 'Phan loai', 'Tin nhan', 'Danh ba', 'HAVIZ'].indexOf(t) >= 0;
          if (!skip) texts.push(t);
        }
      }
    });
    result.textContent = '--- ' + texts.length + ' elements ---\n\n' + texts.slice(-15).join('\n\n');
    result.scrollTop = result.scrollHeight;
  });

  // ---- Send message via Agent ----
  document.getElementById('hv-send').addEventListener('click', function () {
    var msg = document.getElementById('hv-msg').value.trim();
    if (!msg) { result.textContent = 'Nhap tin nhan truoc'; return; }

    if (!tokenInput.value) {
      result.textContent = 'No token set — save a token first.';
      return;
    }

    // Focus Zalo input before sending
    var editables = document.querySelectorAll('[contenteditable="true"]');
    for (var i = 0; i < editables.length; i++) {
      if (!panel.contains(editables[i]) && editables[i].offsetHeight > 0) {
        editables[i].click();
        editables[i].focus();
        break;
      }
    }

    result.textContent = 'Dang gui...';

    fetch(AGENT + '/api/send', {
      method: 'POST',
      headers: authHeaders(),
      body: JSON.stringify({ message: msg, app: 'Google Chrome' }),
    })
      .then(function (r) { return r.json(); })
      .then(function (d) {
        if (d.ok) {
          result.textContent = 'Da gui: "' + msg + '"';
          document.getElementById('hv-msg').value = '';
        } else {
          result.textContent = 'Loi: ' + d.error;
        }
      })
      .catch(function () {
        result.textContent = 'Agent offline.';
      });
  });

  // Enter in textarea triggers send
  document.getElementById('hv-msg').addEventListener('keydown', function (e) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      document.getElementById('hv-send').click();
    }
  });
})();
