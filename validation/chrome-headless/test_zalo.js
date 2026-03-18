/**
 * Test chrome-headless-shell with Zalo Web (chat.zalo.me)
 *
 * Tests: page load, JS execution, DOM extraction, memory usage, load time
 */

const puppeteer = require('puppeteer-core');
const { execSync, spawn } = require('child_process');
const path = require('path');

const CHROME_HEADLESS_PATH = '/tmp/chrome-headless-shell-mac-arm64/chrome-headless-shell';
const ZALO_URL = 'https://chat.zalo.me';
const CDP_PORT = 9222;

function getProcessMemoryRSS(pid) {
  try {
    // macOS: ps returns RSS in KB
    const output = execSync(`ps -o rss= -p ${pid}`, { encoding: 'utf-8' }).trim();
    return parseInt(output, 10) * 1024; // convert KB to bytes
  } catch {
    return null;
  }
}

function formatBytes(bytes) {
  if (bytes === null) return 'N/A';
  const mb = bytes / (1024 * 1024);
  return `${mb.toFixed(1)} MB`;
}

async function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function main() {
  console.log('=== chrome-headless-shell + Zalo Web Test ===\n');
  console.log(`Binary: ${CHROME_HEADLESS_PATH}`);
  console.log(`Target: ${ZALO_URL}`);
  console.log(`Date:   ${new Date().toISOString()}\n`);

  // Check binary exists
  try {
    execSync(`test -x "${CHROME_HEADLESS_PATH}"`);
  } catch {
    console.error('ERROR: chrome-headless-shell binary not found or not executable');
    process.exit(1);
  }

  // Get version
  try {
    const version = execSync(`"${CHROME_HEADLESS_PATH}" --version 2>/dev/null || echo "unknown"`, { encoding: 'utf-8' }).trim();
    console.log(`Version: ${version}`);
  } catch {
    console.log('Version: unknown');
  }

  // Launch chrome-headless-shell
  console.log('\n--- Launching chrome-headless-shell ---');
  const chromeArgs = [
    `--remote-debugging-port=${CDP_PORT}`,
    '--no-sandbox',
    '--disable-gpu',
    '--disable-dev-shm-usage',
    '--disable-extensions',
    '--disable-background-networking',
    '--no-first-run',
    '--disable-sync',
    '--window-size=1280,720',
  ];

  const chromeProcess = spawn(CHROME_HEADLESS_PATH, chromeArgs, {
    stdio: ['ignore', 'pipe', 'pipe'],
  });

  const chromePid = chromeProcess.pid;
  console.log(`Chrome PID: ${chromePid}`);

  // Wait for CDP to be ready
  await sleep(2000);

  // Measure memory before navigation
  const memBefore = getProcessMemoryRSS(chromePid);
  console.log(`Memory (before nav): ${formatBytes(memBefore)}`);

  let browser;
  try {
    // Connect via puppeteer-core
    console.log('\n--- Connecting via puppeteer-core ---');
    browser = await puppeteer.connect({
      browserURL: `http://127.0.0.1:${CDP_PORT}`,
    });
    console.log('Connected to browser');

    const page = await browser.newPage();
    await page.setViewport({ width: 1280, height: 720 });
    await page.setUserAgent(
      'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/146.0.0.0 Safari/537.36'
    );

    // Navigate and measure load time
    console.log('\n--- Navigating to Zalo Web ---');
    const loadStart = Date.now();

    const response = await page.goto(ZALO_URL, {
      waitUntil: 'networkidle2',
      timeout: 60000,
    });

    const loadTime = Date.now() - loadStart;
    console.log(`HTTP Status: ${response.status()}`);
    console.log(`Load time (networkidle2): ${loadTime} ms`);

    // Wait extra for JS frameworks to render
    await sleep(3000);

    // Extract page data
    console.log('\n--- Page Analysis ---');
    const pageData = await page.evaluate(() => {
      const title = document.title;
      const allElements = document.querySelectorAll('*');
      const elementCount = allElements.length;

      // Collect unique CSS classes
      const classSet = new Set();
      allElements.forEach(el => {
        el.classList.forEach(c => classSet.add(c));
      });

      // Get visible text snippets
      const textNodes = [];
      const walker = document.createTreeWalker(
        document.body,
        NodeFilter.SHOW_TEXT,
        {
          acceptNode: (node) => {
            const text = node.textContent.trim();
            if (!text || text.length < 3) return NodeFilter.FILTER_REJECT;
            const parent = node.parentElement;
            if (!parent) return NodeFilter.FILTER_REJECT;
            const style = window.getComputedStyle(parent);
            if (style.display === 'none' || style.visibility === 'hidden') return NodeFilter.FILTER_REJECT;
            return NodeFilter.FILTER_ACCEPT;
          }
        }
      );
      let node;
      while ((node = walker.nextNode()) && textNodes.length < 50) {
        const text = node.textContent.trim();
        if (text.length > 0) textNodes.push(text.substring(0, 100));
      }

      // Body HTML size
      const bodyHTML = document.body ? document.body.innerHTML : '';
      const bodySize = new Blob([bodyHTML]).size;

      // Check for login / chat elements
      const loginIndicators = {
        hasQRCode: !!document.querySelector('canvas, [class*="qr"], [class*="QR"], img[src*="qr"]'),
        hasLoginForm: !!document.querySelector('[class*="login"], [class*="Login"], form'),
        hasPhoneInput: !!document.querySelector('input[type="tel"], input[placeholder*="phone"], input[placeholder*="Phone"]'),
        hasChatList: !!document.querySelector('[class*="chat-list"], [class*="ChatList"], [class*="conversation"]'),
        hasMsgInput: !!document.querySelector('[class*="msg-input"], [class*="compose"], [contenteditable="true"]'),
      };

      // Top-level tag structure
      const bodyChildren = document.body ? Array.from(document.body.children).map(el => ({
        tag: el.tagName,
        id: el.id || '',
        classes: Array.from(el.classList).join(' '),
      })).slice(0, 10) : [];

      // CSS class samples (first 30)
      const classSamples = Array.from(classSet).slice(0, 30);

      return {
        title,
        elementCount,
        bodySize,
        textNodes,
        loginIndicators,
        bodyChildren,
        classSamples,
        url: window.location.href,
      };
    });

    console.log(`URL:           ${pageData.url}`);
    console.log(`Title:         "${pageData.title}"`);
    console.log(`Element count: ${pageData.elementCount}`);
    console.log(`Body HTML:     ${formatBytes(pageData.bodySize)}`);

    console.log('\n--- Login/Chat Element Detection ---');
    for (const [key, val] of Object.entries(pageData.loginIndicators)) {
      console.log(`  ${key}: ${val}`);
    }

    console.log('\n--- Top-level body children ---');
    pageData.bodyChildren.forEach((child, i) => {
      console.log(`  [${i}] <${child.tag}> id="${child.id}" class="${child.classes}"`);
    });

    console.log('\n--- CSS class samples ---');
    console.log(`  Total unique classes: ${pageData.classSamples.length}+`);
    console.log(`  ${pageData.classSamples.join(', ')}`);

    console.log('\n--- Visible text snippets (first 20) ---');
    pageData.textNodes.slice(0, 20).forEach((t, i) => {
      console.log(`  [${i}] ${t}`);
    });

    // Memory after page load
    const memAfter = getProcessMemoryRSS(chromePid);
    console.log(`\n--- Resource Usage ---`);
    console.log(`Memory (after nav):  ${formatBytes(memAfter)}`);
    if (memBefore && memAfter) {
      console.log(`Memory delta:        ${formatBytes(memAfter - memBefore)}`);
    }

    // Also check child processes
    try {
      const psOutput = execSync(
        `ps -o pid,rss,command -p $(pgrep -P ${chromePid}) 2>/dev/null || echo "no children"`,
        { encoding: 'utf-8' }
      ).trim();
      console.log(`\nChild processes:\n${psOutput}`);
    } catch {
      console.log('Child processes: none detected');
    }

    // Total RSS of all chrome processes
    try {
      const totalRSS = execSync(
        `ps aux | grep chrome-headless-shell | grep -v grep | awk '{sum += $6} END {print sum}'`,
        { encoding: 'utf-8' }
      ).trim();
      if (totalRSS && totalRSS !== '0') {
        console.log(`Total RSS (all chrome procs): ${formatBytes(parseInt(totalRSS, 10) * 1024)}`);
      }
    } catch {}

    // Take a screenshot for reference
    const screenshotPath = path.join(__dirname, 'zalo_screenshot.png');
    await page.screenshot({ path: screenshotPath, fullPage: false });
    console.log(`\nScreenshot saved: ${screenshotPath}`);

    // Summary
    console.log('\n=== SUMMARY ===');
    console.log(`Status:        OK`);
    console.log(`HTTP:          ${response.status()}`);
    console.log(`Load time:     ${loadTime} ms`);
    console.log(`Title:         "${pageData.title}"`);
    console.log(`DOM elements:  ${pageData.elementCount}`);
    console.log(`Body HTML:     ${formatBytes(pageData.bodySize)}`);
    console.log(`Memory (RSS):  ${formatBytes(memAfter)}`);
    console.log(`JS rendered:   ${pageData.elementCount > 10 ? 'YES' : 'NO (minimal DOM)'}`);
    console.log(`Login page:    ${pageData.loginIndicators.hasQRCode || pageData.loginIndicators.hasLoginForm ? 'YES' : 'UNKNOWN'}`);
    console.log(`Chat visible:  ${pageData.loginIndicators.hasChatList || pageData.loginIndicators.hasMsgInput ? 'YES' : 'NO (not logged in)'}`);

    await page.close();
  } catch (err) {
    console.error(`\nERROR: ${err.message}`);
    console.error(err.stack);
  } finally {
    if (browser) {
      try { browser.disconnect(); } catch {}
    }
    chromeProcess.kill('SIGTERM');
    console.log('\nChrome process terminated.');
  }
}

main().catch(err => {
  console.error('Fatal error:', err);
  process.exit(1);
});
