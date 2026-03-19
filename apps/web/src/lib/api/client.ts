import { supabase } from '../auth/supabase-client.js';

const API_BASE = '/api';

/** Resolve auth token: agent-injected token takes priority over Supabase session. */
async function getAuthToken(): Promise<string | null> {
  // When embedded in the Agent webview, a token is injected into the window
  const injected = (window as unknown as Record<string, unknown>)['__haviz_agent_token'];
  if (typeof injected === 'string' && injected) return injected;

  // Otherwise use the current Supabase session token
  const { data } = await supabase.auth.getSession();
  return data.session?.access_token ?? null;
}

async function get<T>(path: string): Promise<T> {
  const token = await getAuthToken();
  const headers: Record<string, string> = {};
  if (token) headers['Authorization'] = `Bearer ${token}`;

  const res = await fetch(`${API_BASE}${path}`, { headers });

  if (res.status === 401) {
    await supabase.auth.signOut();
    window.location.reload();
  }

  return res.json();
}

async function post<T>(path: string, body?: unknown): Promise<T> {
  const token = await getAuthToken();
  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
  if (token) headers['Authorization'] = `Bearer ${token}`;

  const res = await fetch(`${API_BASE}${path}`, {
    method: 'POST',
    headers,
    body: body ? JSON.stringify(body) : undefined,
  });

  if (res.status === 401) {
    await supabase.auth.signOut();
    window.location.reload();
  }

  return res.json();
}

export const api = {
  status: () => get<{ ok: boolean; agent: string; version: string }>('/status'),

  conversations: () => get<{ ok: boolean; conversations: Conversation[] }>('/conversations'),
  messages: (convId: string, limit = 50) =>
    get<{ ok: boolean; messages: Message[] }>(`/conversations/${convId}/messages?limit=${limit}`),
  markRead: (convId: string) => post(`/conversations/${convId}/read`),

  drafts: () => get<{ ok: boolean; drafts: Draft[] }>('/drafts'),
  approveDraft: (id: string, opts?: { to?: string; edited_content?: string }) =>
    post(`/drafts/${id}/approve`, opts),
  rejectDraft: (id: string) => post(`/drafts/${id}/reject`),

  templates: () => get<{ ok: boolean; templates: Template[] }>('/templates'),
  createTemplate: (data: { name: string; content: string; category?: string; match_patterns?: string[] }) =>
    post('/templates', data),

  send: (to: string, message: string) => post('/send', { to, message }),

  zalo: {
    conversations: () => get<ZaloData>('/zalo/conversations'),
    search: (query: string) => post<{ ok: boolean; conversations?: ZaloConv[] }>('/zalo/search', { query }),
    open: (index: number) => post('/zalo/open', { index }),
    send: (message: string) => post('/zalo/send', { message }),
    searchAndSend: (to: string, message: string) => post('/zalo/search-and-send', { to, message }),
    messages: () => get<{ ok: boolean; messages: any[] }>('/zalo/messages'),
  },

  ai: {
    draft: (messages: { sender: string; content: string; direction: string }[], orgContext?: string) =>
      post<{ ok: boolean; draft?: string; error?: string }>('/ai/draft', { messages, org_context: orgContext }),
  },

  screenshot: () => fetch(`${API_BASE}/screenshot`).then(r => r.blob()),
};

// Types
export interface Conversation {
  id: string;
  contact_name: string;
  channel_type: string;
  last_message_at: string | null;
  last_message_preview: string | null;
  unread_count: number;
}

export interface Message {
  id: string;
  conversation_id: string;
  direction: string;
  sender_name: string;
  content: string;
  zalo_timestamp: string;
  created_at: string;
}

export interface Draft {
  id: string;
  conversation_id: string;
  content: string;
  status: string;
  created_at: string;
}

export interface Template {
  id: string;
  name: string;
  content: string;
  category: string | null;
  match_patterns: string[];
  usage_count: number;
}

export interface ZaloConv {
  name: string;
  time: string;
  sender: string;
  preview: string;
}

export interface ZaloData {
  browser: string;
  window_title: string;
  conversations: ZaloConv[];
  total_text_elements: number;
}
