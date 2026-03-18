// Simplified Lightpanda + Zalo Web test using CDP directly
const puppeteer = require('puppeteer-core');
const { spawn } = require('child_process');

const LP_PATH = '/tmp/lightpanda';

async function sleep(ms) { return new Promise(r => setTimeout(r, ms)); }

async function main() {
  console.log('=== Lightpanda × Zalo Web — Simple Test ===\n');

  // Start Lightpanda
  const lp = spawn(LP_PATH, ['serve', '--port', '9223', '--timeout', '60', '--http_timeout', '20000', '--log_level', 'warn']);
  lp.stderr.on('data', d => process.stderr.write(d));
  await sleep(2000);

  try {
    const browser = await puppeteer.connect({
      browserWSEndpoint: 'ws://127.0.0.1:9223/',
    });

    // Get all pages/targets
    const pages = await browser.pages();
    console.log(`Pages available: ${pages.length}`);

    const page = pages[0] || await browser.newPage();

    // Use CDP directly for more control
    const client = await page.createCDPSession();

    // Navigate via CDP
    console.log('Navigating to chat.zalo.me via CDP...');
    const nav = await client.send('Page.navigate', { url: 'https://chat.zalo.me' });
    console.log('Navigation result:', JSON.stringify(nav));

    // Wait for load
    await sleep(5000);

    // Get DOM via CDP
    console.log('\nGetting DOM tree...');
    const doc = await client.send('DOM.getDocument', { depth: 3 });
    console.log(`Root node: ${doc.root.nodeName}, children: ${doc.root.childNodeCount}`);

    // Evaluate JS via Runtime
    console.log('\nExecuting JavaScript...');
    const result = await client.send('Runtime.evaluate', {
      expression: `JSON.stringify({
        title: document.title,
        url: window.location.href,
        bodyLen: document.body ? document.body.innerHTML.length : 0,
        elementCount: document.querySelectorAll('*').length,
        divCount: document.querySelectorAll('div').length,
        hasApp: !!document.getElementById('app'),
        texts: Array.from(document.querySelectorAll('*'))
          .filter(el => el.children.length === 0 && el.textContent.trim())
          .map(el => el.textContent.trim())
          .filter(t => t.length > 1 && t.length < 100)
          .slice(0, 20),
        classes: Array.from(new Set(
          Array.from(document.querySelectorAll('[class]'))
            .flatMap(el => el.className.split(' ').filter(Boolean))
        )).slice(0, 30)
      })`,
      returnByValue: true,
    });

    if (result.result.type === 'string') {
      const data = JSON.parse(result.result.value);
      console.log(`\n=== PAGE STATE ===`);
      console.log(`Title: "${data.title}"`);
      console.log(`URL: ${data.url}`);
      console.log(`Body HTML: ${data.bodyLen} chars`);
      console.log(`Elements: ${data.elementCount} total, ${data.divCount} divs`);
      console.log(`#app exists: ${data.hasApp}`);
      console.log(`\nVisible texts:`);
      data.texts.forEach(t => console.log(`  - "${t}"`));
      console.log(`\nCSS classes (sample):`);
      console.log(`  ${data.classes.join(', ')}`);

      console.log(`\n=== VERDICT ===`);
      console.log(`JS executed: ${data.elementCount > 10 ? 'YES' : 'NO'}`);
      console.log(`Zalo loaded: ${data.bodyLen > 1000 ? 'YES' : 'PARTIAL'}`);
      console.log(`Login page: ${data.title.includes('Đăng nhập') ? 'YES (need QR)' : 'NO'}`);
      console.log(`Chat visible: ${data.texts.some(t => t.includes('Tin nhắn') || t.includes('chat')) ? 'MAYBE' : 'NO (not logged in)'}`);
    } else {
      console.log('Eval result:', JSON.stringify(result.result));
    }

    await browser.disconnect();
  } catch (e) {
    console.error('Error:', e.message);
  } finally {
    lp.kill();
    console.log('\nDone.');
  }
}

main();
