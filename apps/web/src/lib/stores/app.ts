import { writable } from 'svelte/store';
import type { ZaloConv, Conversation, Message, Draft } from '../api/client';

export const agentStatus = writable<{ ok: boolean; version: string } | null>(null);
export const zaloConversations = writable<ZaloConv[]>([]);
export const conversations = writable<Conversation[]>([]);
export const currentMessages = writable<Message[]>([]);
export const currentConvId = writable<string | null>(null);
export const pendingDrafts = writable<Draft[]>([]);
export const logs = writable<{ time: string; msg: string; type: 'ok' | 'err' }[]>([]);
export const screenshotUrl = writable<string | null>(null);

export function addLog(msg: string, type: 'ok' | 'err' = 'ok') {
  const time = new Date().toLocaleTimeString();
  logs.update(l => [{ time, msg, type }, ...l].slice(0, 50));
}
