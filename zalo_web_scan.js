// Paste vào DevTools Console (F12) trên chat.zalo.me

// 1. Tìm class patterns liên quan đến chat
const classSet = new Set();
document.querySelectorAll('[class]').forEach(el => {
  if (typeof el.className === 'string') {
    el.className.split(' ').forEach(c => {
      if (c.match(/msg|chat|message|bubble|conv|sender|time|avatar|text/i)) {
        classSet.add(c);
      }
    });
  }
});
console.log("=== RELEVANT CLASSES ===");
console.log(Array.from(classSet).sort().join('\n'));

// 2. Brute force: lấy tất cả leaf text nodes
console.log("\n=== TIN NHẮN (last 30 text elements) ===");
const texts = [];
document.querySelectorAll('div, span, p').forEach(el => {
  if (el.children.length === 0) {
    const t = el.innerText?.trim();
    if (t && t.length > 1 && t.length < 1000) {
      texts.push({ text: t, class: el.className?.substring?.(0, 60) || '', tag: el.tagName });
    }
  }
});
texts.slice(-30).forEach((item, i) => {
  console.log(`[${item.class}] "${item.text}"`);
});
console.log(`\nTotal: ${texts.length} text elements`);
