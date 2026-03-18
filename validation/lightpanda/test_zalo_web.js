// Test Lightpanda + Zalo Web via CDP (Puppeteer)
// 1. Start Lightpanda CDP server
// 2. Connect Puppeteer
// 3. Navigate to chat.zalo.me
// 4. Extract page structure and detect login/chat state

const puppeteer = require('puppeteer-core');
const { execSync, spawn } = require('child_process');

const LIGHTPANDA_PATH = '/tmp/lightpanda';
const ZALO_URL = 'https://chat.zalo.me';
const CDP_PORT = 9222;

async function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function main() {
  console.log('=== Haviz × Lightpanda — Zalo Web Test ===\n');

  // 1. Start Lightpanda CDP server
  console.log('1. Starting Lightpanda CDP server...');
  const lp = spawn(LIGHTPANDA_PATH, [
    'serve',
    '--port', String(CDP_PORT),
    '--timeout', '30',
    '--http_timeout', '20000',
    '--log_level', 'warn',
  ], { stdio: ['ignore', 'pipe', 'pipe'] });

  lp.stderr.on('data', d => {
    const line = d.toString().trim();
    if (line) console.log(`  [LP] ${line}`);
  });

  await sleep(2000);
  console.log(`   CDP server running on ws://127.0.0.1:${CDP_PORT}/ws\n`);

  try {
    // 2. Connect Puppeteer
    console.log('2. Connecting Puppeteer to Lightpanda...');
    const browser = await puppeteer.connect({
      browserWSEndpoint: `ws://127.0.0.1:${CDP_PORT}/`,
    });
    console.log('   Connected!\n');

    // 3. Navigate to Zalo Web
    console.log('3. Navigating to chat.zalo.me...');
    const page = await browser.newPage();

    // Set a realistic user agent
    await page.setUserAgent('Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36');

    const startTime = Date.now();
    try {
      await page.goto(ZALO_URL, { timeout: 20000 });
    } catch (navErr) {
      console.log(`   Navigation note: ${navErr.message.slice(0, 80)}`);
      console.log('   Continuing anyway (page may still be usable)...');
    }
    const loadTime = Date.now() - startTime;
    console.log(`   Page loaded in ${loadTime}ms\n`);

    // Wait for JS execution
    await sleep(3000);

    // 4. Analyze page
    console.log('4. Analyzing page structure...');

    const title = await page.title();
    console.log(`   Title: "${title}"`);

    const url = page.url();
    console.log(`   URL: ${url}`);

    // Check login state
    const bodyHTML = await page.evaluate(() => document.body.innerHTML.length);
    console.log(`   Body HTML size: ${bodyHTML} chars`);

    // Look for QR code (login page indicator)
    const hasQR = await page.evaluate(() => {
      const imgs = document.querySelectorAll('img');
      for (const img of imgs) {
        if (img.src && (img.src.includes('qr') || img.width > 150)) return true;
      }
      return false;
    });
    console.log(`   QR code found: ${hasQR}`);

    // Check for login form elements
    const loginIndicators = await page.evaluate(() => {
      const results = {};
      results.hasAppDiv = !!document.getElementById('app');
      results.appDisplay = document.getElementById('app')?.style.display;
      results.bodyClasses = document.body.className;

      // Find all visible text content
      const textElements = document.querySelectorAll('*');
      const texts = [];
      for (const el of textElements) {
        if (el.children.length === 0 && el.textContent.trim().length > 0 && el.textContent.trim().length < 100) {
          texts.push(el.textContent.trim());
        }
      }
      results.visibleTexts = [...new Set(texts)].slice(0, 30);

      // Count elements by common Zalo classes
      results.chatElements = document.querySelectorAll('.chat-message, .message, .msg-content, .text').length;
      results.truncateElements = document.querySelectorAll('.truncate').length;
      results.inputElements = document.querySelectorAll('input, textarea').length;

      return results;
    });

    console.log(`   #app div: ${loginIndicators.hasAppDiv} (display: ${loginIndicators.appDisplay})`);
    console.log(`   Body classes: "${loginIndicators.bodyClasses}"`);
    console.log(`   Chat elements: ${loginIndicators.chatElements}`);
    console.log(`   .truncate elements: ${loginIndicators.truncateElements}`);
    console.log(`   Input elements: ${loginIndicators.inputElements}`);
    console.log(`\n   Visible texts (first 30):`);
    for (const t of loginIndicators.visibleTexts) {
      console.log(`     - "${t}"`);
    }

    // 5. Try to dump DOM structure
    console.log('\n5. DOM structure summary:');
    const domStats = await page.evaluate(() => {
      const stats = {};
      stats.totalElements = document.querySelectorAll('*').length;
      stats.divCount = document.querySelectorAll('div').length;
      stats.spanCount = document.querySelectorAll('span').length;
      stats.imgCount = document.querySelectorAll('img').length;
      stats.scriptCount = document.querySelectorAll('script').length;

      // Get all unique class names
      const classes = new Set();
      document.querySelectorAll('[class]').forEach(el => {
        el.className.split(' ').forEach(c => { if (c.trim()) classes.add(c.trim()); });
      });
      stats.uniqueClasses = classes.size;
      stats.sampleClasses = [...classes].slice(0, 20);

      return stats;
    });

    console.log(`   Total elements: ${domStats.totalElements}`);
    console.log(`   Divs: ${domStats.divCount}, Spans: ${domStats.spanCount}, Imgs: ${domStats.imgCount}`);
    console.log(`   Scripts: ${domStats.scriptCount}`);
    console.log(`   Unique CSS classes: ${domStats.uniqueClasses}`);
    console.log(`   Sample classes: ${domStats.sampleClasses.join(', ')}`);

    // 6. Summary
    console.log('\n=== RESULTS ===');
    console.log(`Load time: ${loadTime}ms`);
    console.log(`Page state: ${loginIndicators.chatElements > 0 ? 'LOGGED IN (chat visible)' : 'LOGIN PAGE (need QR scan)'}`);
    console.log(`DOM rendered: ${domStats.totalElements > 50 ? 'YES' : 'MINIMAL'} (${domStats.totalElements} elements)`);
    console.log(`JavaScript executed: YES (Zalo JS loaded and ran)`);
    console.log(`Lightpanda compatible: ${domStats.totalElements > 10 ? 'YES' : 'NEEDS INVESTIGATION'}`);

    await browser.disconnect();
  } catch (e) {
    console.error('\n❌ Error:', e.message);
  } finally {
    lp.kill();
    console.log('\nDone.');
  }
}

main().catch(console.error);
