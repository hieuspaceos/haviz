// Test Lightpanda + Playwright (thay Puppeteer)
// Playwright handle CDP khác: ít messages hơn, frame handling khác
const { chromium } = require('playwright-core');
const { spawn } = require('child_process');

const LP_PATH = '/tmp/lightpanda';

async function sleep(ms) { return new Promise(r => setTimeout(r, ms)); }

async function main() {
  console.log('=== Lightpanda × Playwright — Zalo Web Test ===\n');

  // Start Lightpanda CDP server
  console.log('1. Starting Lightpanda...');
  const lp = spawn(LP_PATH, [
    'serve', '--port', '9224', '--timeout', '60',
    '--http_timeout', '20000', '--log_level', 'warn'
  ]);
  lp.stderr.on('data', d => {
    const line = d.toString().trim();
    if (line && !line.includes('telemetry')) console.log(`  [LP] ${line}`);
  });
  await sleep(2000);

  try {
    // Connect Playwright via CDP
    console.log('2. Connecting Playwright via CDP...');
    const browser = await chromium.connectOverCDP('http://127.0.0.1:9224');
    console.log('   Connected!\n');

    const contexts = browser.contexts();
    console.log(`   Browser contexts: ${contexts.length}`);

    // Get or create page
    let page;
    if (contexts.length > 0 && contexts[0].pages().length > 0) {
      page = contexts[0].pages()[0];
    } else {
      const context = contexts[0] || await browser.newContext();
      page = await context.newPage();
    }

    // Navigate
    console.log('3. Navigating to chat.zalo.me...');
    const start = Date.now();
    try {
      await page.goto('https://chat.zalo.me', { timeout: 20000, waitUntil: 'load' });
      console.log(`   Loaded in ${Date.now() - start}ms`);
    } catch (e) {
      console.log(`   Navigation: ${e.message.slice(0, 100)}`);
      console.log('   Trying domcontentloaded...');
      try {
        await page.goto('https://chat.zalo.me', { timeout: 20000, waitUntil: 'domcontentloaded' });
        console.log(`   Loaded (domcontentloaded) in ${Date.now() - start}ms`);
      } catch (e2) {
        console.log(`   Navigation fallback: ${e2.message.slice(0, 100)}`);
        console.log('   Trying commit...');
        try {
          await page.goto('https://chat.zalo.me', { timeout: 20000, waitUntil: 'commit' });
          console.log(`   Loaded (commit) in ${Date.now() - start}ms`);
        } catch (e3) {
          console.log(`   All waitUntil failed: ${e3.message.slice(0, 100)}`);
        }
      }
    }

    // Wait for JS
    await sleep(5000);

    // Analyze
    console.log('\n4. Analyzing page...');
    try {
      const title = await page.title();
      console.log(`   Title: "${title}"`);
      const url = page.url();
      console.log(`   URL: ${url}`);

      const data = await page.evaluate(() => {
        return {
          bodyLen: document.body ? document.body.innerHTML.length : 0,
          elements: document.querySelectorAll('*').length,
          divs: document.querySelectorAll('div').length,
          spans: document.querySelectorAll('span').length,
          scripts: document.querySelectorAll('script').length,
          hasApp: !!document.getElementById('app'),
          texts: Array.from(document.querySelectorAll('*'))
            .filter(el => el.children.length === 0 && el.textContent.trim())
            .map(el => el.textContent.trim())
            .filter(t => t.length > 1 && t.length < 100)
            .slice(0, 20),
          classes: Array.from(new Set(
            Array.from(document.querySelectorAll('[class]'))
              .flatMap(el => el.className.split ? el.className.split(' ').filter(Boolean) : [])
          )).slice(0, 30),
        };
      });

      console.log(`   Body HTML: ${data.bodyLen} chars`);
      console.log(`   Elements: ${data.elements} total (${data.divs} divs, ${data.spans} spans)`);
      console.log(`   #app: ${data.hasApp}`);
      console.log(`   Scripts loaded: ${data.scripts}`);
      console.log(`\n   Visible texts:`);
      data.texts.forEach(t => console.log(`     "${t}"`));
      console.log(`\n   CSS classes: ${data.classes.join(', ')}`);

      // Check for chat-specific elements
      const chatCheck = await page.evaluate(() => {
        return {
          chatMessages: document.querySelectorAll('.chat-message, .msg-content, [class*="message"]').length,
          truncate: document.querySelectorAll('.truncate').length,
          inputs: document.querySelectorAll('input, textarea, [contenteditable]').length,
          imgs: document.querySelectorAll('img').length,
          loginForm: !!document.querySelector('[class*="login"], [class*="qr"]'),
        };
      });
      console.log(`\n   Chat elements: ${chatCheck.chatMessages}`);
      console.log(`   .truncate: ${chatCheck.truncate}`);
      console.log(`   Inputs: ${chatCheck.inputs}`);
      console.log(`   Images: ${chatCheck.imgs}`);
      console.log(`   Login/QR form: ${chatCheck.loginForm}`);

      console.log('\n=== VERDICT ===');
      console.log(`Playwright + Lightpanda: WORKING`);
      console.log(`JS executed: ${data.elements > 10 ? 'YES' : 'NO'}`);
      console.log(`Page state: ${data.texts.some(t => t.includes('Đăng nhập') || t.includes('login')) ? 'LOGIN PAGE' : 'UNKNOWN'}`);

    } catch (evalErr) {
      console.log(`   Evaluate error: ${evalErr.message}`);
    }

    await browser.close();
  } catch (e) {
    console.error(`\nError: ${e.message}`);
  } finally {
    lp.kill();
    console.log('\nDone.');
  }
}

main();
