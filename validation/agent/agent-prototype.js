// Haviz Agent — Tự tìm user + gửi tin nhắn
// Chạy: node agent.js

const http = require('http');
const { execSync } = require('child_process');

const PORT = 9999;

function esc(str) {
  return str.replace(/\\/g, '\\\\').replace(/"/g, '\\"');
}

function sendToZaloDesktop(to, message) {
  // Step 1: Focus Zalo + mở search + paste tên user (clipboard cho tiếng Việt)
  execSync(`osascript -e 'set the clipboard to "${esc(to)}"'`);
  execSync(`osascript -e '
    tell application "Zalo" to activate
    delay 0.5
    tell application "System Events"
      keystroke "f" using command down
      delay 0.5
      keystroke "a" using command down
      delay 0.1
      keystroke "v" using command down
      delay 1
      key code 36
      delay 0.5
    end tell
  '`);

  // Step 2: Paste tin nhắn + Enter
  execSync(`osascript -e 'set the clipboard to "${esc(message)}"'`);
  execSync(`osascript -e '
    tell application "System Events"
      keystroke "v" using command down
      delay 0.3
      key code 36
    end tell
  '`);
}

function sendToZaloWeb(to, message, browser) {
  const browserApp = browser || 'Google Chrome';
  const processName = browserApp === 'Safari' ? 'Safari' : browserApp;

  // Step 1: Copy tên user vào clipboard + Focus browser + search
  execSync(`osascript -e 'set the clipboard to "${esc(to)}"'`);
  execSync(`osascript -e '
    tell application "${browserApp}" to activate
    delay 0.5
    tell application "System Events"
      tell process "${processName}"
        set frontmost to true
        set winPos to position of window 1
        set winSize to size of window 1
      end tell
      -- Click vào ô tìm kiếm (góc trên trái, dưới avatar)
      set xSearch to (item 1 of winPos) + 200
      set ySearch to (item 2 of winPos) + 180
      click at {xSearch, ySearch}
      delay 0.3
      -- Xóa text cũ + paste tên user (clipboard cho tiếng Việt)
      keystroke "a" using command down
      delay 0.1
      keystroke "v" using command down
      delay 1.5
      -- Enter chọn kết quả đầu tiên
      key code 36
      delay 0.8
      -- Click vào ô nhập tin nhắn (cuối cửa sổ)
      set xMsg to (item 1 of winPos) + (item 1 of winSize) / 2
      set yMsg to (item 2 of winPos) + (item 2 of winSize) - 30
      click at {xMsg, yMsg}
      delay 0.3
    end tell
  '`);

  // Step 2: Paste tin nhắn + Enter
  execSync(`osascript -e 'set the clipboard to "${esc(message)}"'`);
  execSync(`osascript -e '
    tell application "System Events"
      keystroke "v" using command down
      delay 0.3
      key code 36
    end tell
  '`);
}

const server = http.createServer((req, res) => {
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

  if (req.method === 'OPTIONS') { res.writeHead(200); res.end(); return; }

  if (req.method === 'POST' && req.url === '/send') {
    let body = '';
    req.on('data', chunk => body += chunk);
    req.on('end', () => {
      try {
        const { to, message, app } = JSON.parse(body);
        const target = app || 'Zalo';

        if (!message) throw new Error('Thiếu message');

        const webBrowsers = ['Google Chrome', 'Safari', 'Arc', 'Microsoft Edge', 'Firefox', 'Brave Browser'];
        const isWebBrowser = webBrowsers.includes(target);

        if (to) {
          // Có chỉ định người nhận → tự tìm + gửi
          if (isWebBrowser) {
            sendToZaloWeb(to, message, target);
          } else {
            sendToZaloDesktop(to, message);
          }
          console.log(`✅ Gửi đến "${to}" (${target}): "${message}"`);
        } else {
          // Không có to → gửi vào chat đang mở
          execSync(`osascript -e 'set the clipboard to "${esc(message)}"'`);
          execSync(`osascript -e '
            tell application "${target === "Google Chrome" ? "Google Chrome" : "Zalo"}" to activate
            delay 0.3
            tell application "System Events"
              ${target === 'Google Chrome' ? `
              tell process "Google Chrome"
                set winPos to position of window 1
                set winSize to size of window 1
              end tell
              set xMsg to (item 1 of winPos) + (item 1 of winSize) / 2
              set yMsg to (item 2 of winPos) + (item 2 of winSize) - 30
              click at {xMsg, yMsg}
              delay 0.3` : ''}
              keystroke "v" using command down
              delay 0.3
              key code 36
            end tell
          '`);
          console.log(`✅ Gửi (${target}): "${message}"`);
        }

        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ ok: true, to, message }));
      } catch (e) {
        console.log('❌ Lỗi:', e.message);
        res.writeHead(500, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ ok: false, error: e.message }));
      }
    });
    return;
  }

  if (req.method === 'GET' && req.url === '/status') {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ ok: true, agent: 'haviz', version: '0.2' }));
    return;
  }

  res.writeHead(404);
  res.end('Not found');
});

server.listen(PORT, () => {
  console.log(`🚀 Haviz Agent v0.2 running on http://localhost:${PORT}`);
  console.log('');
  console.log('POST /send:');
  console.log('  { "to": "Tên user", "message": "Nội dung", "app": "Zalo" }');
  console.log('  { "to": "Tên user", "message": "Nội dung", "app": "Google Chrome" }');
  console.log('  { "message": "Nội dung" }  ← gửi vào chat đang mở');
});
