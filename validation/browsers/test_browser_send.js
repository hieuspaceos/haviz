// Test gửi tin nhắn qua Agent trên nhiều browser
// Chạy: node test_browser_send.js
// Yêu cầu: Agent (agent-prototype.js) đang chạy trên localhost:9999

const http = require('http');

const browsers = [
  { name: 'Google Chrome', app: 'Google Chrome' },
  { name: 'Safari', app: 'Safari' },
  { name: 'Arc', app: 'Arc' },
  { name: 'Microsoft Edge', app: 'Microsoft Edge' },
  { name: 'Firefox', app: 'Firefox' },
];

async function sendTest(browserApp, message) {
  return new Promise((resolve, reject) => {
    const data = JSON.stringify({ message, app: browserApp });
    const req = http.request({
      hostname: 'localhost', port: 9999, path: '/send',
      method: 'POST',
      headers: { 'Content-Type': 'application/json' }
    }, (res) => {
      let body = '';
      res.on('data', chunk => body += chunk);
      res.on('end', () => resolve(JSON.parse(body)));
    });
    req.on('error', reject);
    req.write(data);
    req.end();
  });
}

async function main() {
  console.log('=== TEST GỬI TIN NHẮN - MULTI BROWSER ===\n');

  for (const browser of browsers) {
    try {
      console.log(`🔍 ${browser.name}...`);
      const result = await sendTest(browser.app, `Test từ ${browser.name}`);
      if (result.ok) {
        console.log(`   ✅ Gửi thành công`);
      } else {
        console.log(`   ❌ Lỗi: ${result.error}`);
      }
    } catch (e) {
      console.log(`   ⏭  Bỏ qua (Agent offline hoặc browser chưa mở)`);
    }
  }
}

main();
