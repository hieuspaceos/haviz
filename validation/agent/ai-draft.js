// Haviz AI Draft — Groq + Llama 4 Scout
// Tích hợp vào Agent: nhận tin nhắn → AI draft reply → user approve → gửi

const http = require('http');
const https = require('https');
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Load env
const envPath = path.join(__dirname, '../../.env.local');
const env = {};
fs.readFileSync(envPath, 'utf8').split('\n').forEach(line => {
  const [key, ...val] = line.split('=');
  if (key && !key.startsWith('#')) env[key.trim()] = val.join('=').trim();
});

const GROQ_API_KEY = env.GROQ_API_KEY;
const PORT = 9999;

if (!GROQ_API_KEY) {
  console.error('❌ GROQ_API_KEY không tìm thấy trong .env.local');
  process.exit(1);
}

function esc(str) {
  return str.replace(/\\/g, '\\\\').replace(/"/g, '\\"');
}

// === GROQ API CALL ===
function callGroq(messages, options = {}) {
  return new Promise((resolve, reject) => {
    const data = JSON.stringify({
      model: options.model || 'meta-llama/llama-4-scout-17b-16e-instruct',
      messages,
      temperature: options.temperature || 0.7,
      max_tokens: options.max_tokens || 300,
    });

    const req = https.request({
      hostname: 'api.groq.com',
      path: '/openai/v1/chat/completions',
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${GROQ_API_KEY}`,
        'Content-Type': 'application/json',
      }
    }, (res) => {
      let body = '';
      res.on('data', chunk => body += chunk);
      res.on('end', () => {
        try {
          const json = JSON.parse(body);
          if (json.choices && json.choices[0]) {
            resolve(json.choices[0].message.content);
          } else {
            reject(new Error(json.error?.message || 'Groq API error'));
          }
        } catch (e) { reject(e); }
      });
    });
    req.on('error', reject);
    req.write(data);
    req.end();
  });
}

// === AI DRAFT ===
async function generateDraft(conversation, orgContext) {
  const systemPrompt = `Bạn là trợ lý bán hàng AI cho doanh nghiệp Việt Nam. Vai trò:
- Soạn tin nhắn trả lời khách hàng bằng tiếng Việt tự nhiên
- Phong cách: thân thiện, chuyên nghiệp, phù hợp văn hóa Việt Nam
- Dùng emoji vừa phải, xưng hô phù hợp (anh/chị/em)
- Trả lời ngắn gọn, đúng trọng tâm
- KHÔNG giải thích, chỉ trả lời nội dung tin nhắn

${orgContext ? 'Bối cảnh doanh nghiệp: ' + orgContext : ''}

Chỉ trả về nội dung tin nhắn reply, không giải thích gì thêm.`;

  const messages = [
    { role: 'system', content: systemPrompt },
    ...conversation.map(msg => ({
      role: msg.direction === 'inbound' ? 'user' : 'assistant',
      content: `${msg.sender}: ${msg.content}`
    }))
  ];

  return callGroq(messages);
}

// === SEND VIA ZALO ===
function sendToZalo(to, message, app) {
  execSync(`osascript -e 'set the clipboard to "${esc(to)}"'`);
  execSync(`osascript -e '
    tell application "${app || 'Zalo'}" to activate
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

  execSync(`osascript -e 'set the clipboard to "${esc(message)}"'`);
  execSync(`osascript -e '
    tell application "System Events"
      keystroke "v" using command down
      delay 0.3
      key code 36
    end tell
  '`);
}

// === PENDING DRAFTS ===
let pendingDrafts = {};
let draftCounter = 0;

// === HTTP SERVER ===
const server = http.createServer(async (req, res) => {
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'POST, GET, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

  if (req.method === 'OPTIONS') { res.writeHead(200); res.end(); return; }

  let body = '';
  req.on('data', chunk => body += chunk);

  await new Promise(resolve => req.on('end', resolve));

  try {
    // === AI DRAFT — tạo draft reply ===
    if (req.method === 'POST' && req.url === '/ai/draft') {
      const { conversation, orgContext } = JSON.parse(body);
      // conversation: [{ sender, content, direction: "inbound"|"outbound" }]

      if (!conversation || !conversation.length) {
        res.writeHead(400, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ ok: false, error: 'Thiếu conversation' }));
        return;
      }

      console.log(`🤖 Đang tạo AI draft cho ${conversation.length} tin nhắn...`);
      const draft = await generateDraft(conversation, orgContext);

      draftCounter++;
      const draftId = `draft_${draftCounter}`;
      pendingDrafts[draftId] = {
        id: draftId,
        content: draft,
        conversation,
        status: 'pending',
        createdAt: new Date().toISOString()
      };

      console.log(`✅ Draft #${draftId}: "${draft}"`);

      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ ok: true, draftId, draft }));
      return;
    }

    // === LIST DRAFTS ===
    if (req.method === 'GET' && req.url === '/ai/drafts') {
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ ok: true, drafts: Object.values(pendingDrafts) }));
      return;
    }

    // === APPROVE DRAFT — approve + gửi luôn ===
    if (req.method === 'POST' && req.url.startsWith('/ai/drafts/') && req.url.endsWith('/approve')) {
      const draftId = req.url.split('/')[3];
      const draft = pendingDrafts[draftId];

      if (!draft) {
        res.writeHead(404, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ ok: false, error: 'Draft không tồn tại' }));
        return;
      }

      const { to, app, editedContent } = JSON.parse(body || '{}');
      const messageToSend = editedContent || draft.content;
      const target = app || 'Zalo';

      if (to) {
        sendToZalo(to, messageToSend, target);
        console.log(`✅ Approved + sent draft #${draftId} to "${to}": "${messageToSend}"`);
      }

      draft.status = 'approved';
      draft.approvedAt = new Date().toISOString();

      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ ok: true, draftId, sent: !!to, message: messageToSend }));
      return;
    }

    // === REJECT DRAFT ===
    if (req.method === 'POST' && req.url.startsWith('/ai/drafts/') && req.url.endsWith('/reject')) {
      const draftId = req.url.split('/')[3];
      if (pendingDrafts[draftId]) {
        pendingDrafts[draftId].status = 'rejected';
        console.log(`❌ Rejected draft #${draftId}`);
      }
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ ok: true, draftId }));
      return;
    }

    // === SEND (giữ nguyên từ agent-prototype) ===
    if (req.method === 'POST' && req.url === '/send') {
      const { to, message, app } = JSON.parse(body);
      if (!message) throw new Error('Thiếu message');
      sendToZalo(to || '', message, app || 'Zalo');
      console.log(`✅ Gửi: "${message}"`);
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ ok: true, to, message }));
      return;
    }

    // === STATUS ===
    if (req.method === 'GET' && req.url === '/status') {
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({
        ok: true, agent: 'haviz', version: '0.3',
        ai: { model: 'llama-4-scout-17b-16e-instruct', provider: 'groq' },
        pendingDrafts: Object.keys(pendingDrafts).length
      }));
      return;
    }

    res.writeHead(404);
    res.end('Not found');

  } catch (e) {
    console.log('❌ Lỗi:', e.message);
    res.writeHead(500, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ ok: false, error: e.message }));
  }
});

server.listen(PORT, () => {
  console.log(`🚀 Haviz Agent v0.3 (AI Draft) running on http://localhost:${PORT}`);
  console.log('');
  console.log('Endpoints:');
  console.log('  POST /send                    — gửi tin nhắn');
  console.log('  POST /ai/draft                — tạo AI draft reply');
  console.log('  GET  /ai/drafts               — xem pending drafts');
  console.log('  POST /ai/drafts/:id/approve   — approve + gửi');
  console.log('  POST /ai/drafts/:id/reject    — reject draft');
  console.log('  GET  /status                  — agent status');
});
