# Haviz Code Standards & Conventions

**Version:** 1.0
**Last Updated:** 2026-03-19
**Status:** Active

This document defines coding conventions, patterns, and quality standards across the Haviz monorepo.

## General Principles

1. **Readability** — Code is read 10x more than written
2. **Maintainability** — Future developers should understand intent without extensive comments
3. **Consistency** — Same patterns across entire codebase
4. **Performance** — Optimize where it matters; don't prematurely optimize
5. **Safety** — Fail loudly, handle errors explicitly

## File & Directory Naming

### Rust Files (`agent/`)

**Convention:** `snake_case`

```rust
// ✓ Good
src/message_parser.rs
src/ai.rs
src/channels/zalo_web.rs

// ✗ Bad
src/messageParser.rs
src/AI.rs
src/channels/ZaloWeb.rs
```

**Modules:** Match file structure

```rust
// src/channels/zalo_web.rs
pub mod channels {
    pub mod zalo_web { ... }
}

// Usage: crate::channels::zalo_web::ZaloWebReader
```

### TypeScript/JavaScript Files (`apps/web/`, `extensions/`)

**Convention:** `kebab-case` with descriptive names

```
// ✓ Good
src/lib/api/client.ts
src/lib/components/InboxView.svelte
src/lib/stores/app.ts

// ✗ Bad
src/lib/api/ApiClient.ts
src/lib/components/InboxView/InboxView.svelte
src/lib/store.ts  (too vague)
```

**Exceptions:**
- Component files: `PascalCase.svelte` (Svelte convention)
- Classes: `PascalCase.ts`
- Utilities: `camelCase.ts`

### Test Files

```
// ✓ Good
src/message_parser.test.ts
agent/tests/ai_draft_generation_test.rs

// ✗ Bad
test_message_parser.ts
message_parser_test.ts
```

## Code Organization

### Rust Agent (`agent/src/`)

**Directory Structure:**

```
agent/src/
├── main.rs                 # CLI entry point
├── lib.rs                  # Library root
├── bin/
│   └── haviz_app.rs        # Desktop app entry point
├── server.rs               # Axum HTTP server
├── db.rs                   # SQLite database
├── ai.rs                   # Groq API integration
├── polling.rs              # Message polling loop
├── message_parser.rs       # Parse raw text
├── config.rs               # Configuration
├── channels/
│   ├── mod.rs              # Channel module root
│   ├── traits.rs           # ChannelReader, ChannelSender traits
│   ├── zalo_web.rs         # Zalo Web implementation
│   └── zalo_desktop.rs     # Zalo Desktop implementation
├── platform/
│   ├── mod.rs
│   ├── macos/
│   │   ├── mod.rs
│   │   ├── accessibility.rs  # AX API bindings
│   │   └── automation.rs     # AppleScript automation
│   └── windows/
│       └── mod.rs
└── helpers/                # Utility functions (if extracted)
```

**File Size Limit:** Max 200 lines per file
- If file approaches 200 lines, extract functions to new module
- Example: `db.rs` (331 LOC) should be split into `db/queries.rs`, `db/migrations.rs`

### Web UI (`apps/web/src/`)

**Directory Structure:**

```
src/
├── App.svelte              # Root component
├── main.ts                 # Entry point
├── app.css                 # Global styles
├── lib/
│   ├── api/
│   │   └── client.ts       # REST client
│   ├── components/
│   │   ├── InboxView.svelte   # Main view
│   │   ├── Sidebar.svelte
│   │   ├── Topbar.svelte
│   │   └── LogPanel.svelte
│   └── stores/
│       └── app.ts          # Svelte stores
└── routes/
    └── [if using SvelteKit]
```

**Component Structure:**

```svelte
<!-- ✓ Good structure -->
<script lang="ts">
  // 1. Imports (types first, then components, then stores)
  import type { Message } from '$lib/types';
  import InboxView from './InboxView.svelte';
  import { appStore } from '$lib/stores/app';

  // 2. Exports (define props)
  export let messages: Message[] = [];

  // 3. Reactive declarations
  let filteredMessages = [];
  $: filteredMessages = messages.filter(m => m.read === false);

  // 4. Functions
  function handleSend(msg: string) { ... }

  // 5. Lifecycle
  onMount(() => { ... });
</script>

<!-- Markup -->
<div class="container">
  {#each filteredMessages as msg (msg.id)}
    <MessageItem {msg} />
  {/each}
</div>

<style>
  /* Scoped styles */
  .container { ... }
</style>
```

## Naming Conventions

### Variables & Functions

**Rust:**

```rust
// ✓ Good
let message_count = 42;
let is_online = true;
let user_preferences: UserPrefs;

fn parse_message(raw: &str) -> Result<Message, ParseError> { ... }
fn should_send_message(msg: &Message) -> bool { ... }

// ✗ Bad
let msgCnt = 42;           // camelCase
let message = true;        // ambiguous
let userPrefs: UserPrefs;  // camelCase in Rust

fn parseMessage(...) { }   // camelCase
fn check_msg(...) { }      // vague name
```

**TypeScript/JavaScript:**

```typescript
// ✓ Good
const messageCount = 42;
const isOnline = true;
const userPreferences: UserPrefs;

function parseMessage(raw: string): Message { ... }
function shouldSendMessage(msg: Message): boolean { ... }

// ✗ Bad
const message_count = 42;  // snake_case
const message = true;      // ambiguous
let msgCnt = 42;           // abbreviation

const parseMsg = (...) => {}  // vague
```

### Constants

```rust
// Rust
const MAX_MESSAGE_LENGTH: usize = 4096;
const DEFAULT_POLLING_INTERVAL_MS: u64 = 3000;
const RATE_LIMIT_PER_MINUTE: u32 = 5;
```

```typescript
// TypeScript
export const MAX_MESSAGE_LENGTH = 4096;
export const DEFAULT_POLLING_INTERVAL_MS = 3000;
export const RATE_LIMIT_PER_MINUTE = 5;
```

### Boolean Variables

Always prefix with `is`, `has`, `should`, `can`:

```rust
// ✓ Good
let is_online = true;
let has_unread = true;
let should_send = true;
let can_retry = true;

// ✗ Bad
let online = true;
let unread = true;
let send = true;
let retry = true;
```

## Type Safety

### Rust Types

```rust
// ✓ Use NewTypes for domain concepts
pub struct MessageId(pub String);
pub struct UserId(pub String);
pub struct DraftId(pub u64);

// ✓ Use enums for finite states
pub enum MessageStatus {
    Pending,
    Sent,
    Delivered,
    Failed(String),
}

// ✓ Use Result for fallible operations
pub fn send_message(msg: &Message) -> Result<SendReceipt, SendError> { ... }

// ✗ Avoid stringly-typed domains
pub fn send_message(msg_id: String) -> bool { ... }  // Bad
```

### TypeScript Types

```typescript
// ✓ Good: Explicit types
interface Message {
  id: string;
  senderId: string;
  content: string;
  sentAt: Date;
  status: 'pending' | 'sent' | 'delivered' | 'failed';
}

// ✓ Use discriminated unions
type MessageEvent =
  | { type: 'sent'; messageId: string }
  | { type: 'failed'; error: string }
  | { type: 'delivered'; timestamp: Date };

// ✗ Avoid 'any'
function parseMessage(data: any): Message { ... }  // Bad

// ✗ Avoid overly permissive types
interface Message {
  [key: string]: unknown;  // Bad: not self-documenting
}
```

## Error Handling

### Rust

```rust
// ✓ Good: Use Result types, propagate with ?
pub fn read_messages(conn: &Connection) -> Result<Vec<Message>, DbError> {
    let mut stmt = conn.prepare("SELECT * FROM messages")?;
    let messages = stmt.query_map([], |row| {
        Ok(Message {
            id: row.get(0)?,
            content: row.get(1)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    Ok(messages)
}

// ✓ Use custom error types
#[derive(Debug)]
pub enum ParseError {
    InvalidFormat(String),
    MissingField(String),
}

// ✗ Avoid panic! in library code
fn parse_message(s: &str) -> Message {
    // Don't use panic! here
    panic!("Invalid message");  // Bad
}

// ✗ Avoid ignoring errors
let _ = db.save_message(&msg);  // Bad: silent failure
```

### TypeScript

```typescript
// ✓ Good: Throw with descriptive errors
function parseMessage(raw: string): Message {
  try {
    const data = JSON.parse(raw);
    if (!data.id) throw new Error('Missing required field: id');
    return data as Message;
  } catch (err) {
    throw new Error(`Failed to parse message: ${err.message}`);
  }
}

// ✓ Handle errors explicitly
async function sendMessage(msg: Message): Promise<void> {
  try {
    const response = await apiClient.send(msg);
    if (!response.ok) throw new Error(`Send failed: ${response.status}`);
  } catch (err) {
    logger.error('Send message failed', { error: err, messageId: msg.id });
    throw err;
  }
}

// ✗ Avoid silent failures
const response = await fetch(url).catch(() => null);  // Bad
if (!response) { ... }  // Silent failure
```

## Comments & Documentation

### When to Comment

**✓ Good comments:**

```rust
// Use comments to explain WHY, not WHAT

// We poll every 3s instead of 1s because:
// 1. Zalo AX API has rate limits
// 2. Users rarely need real-time updates
// 3. Saves CPU and battery
const POLLING_INTERVAL_MS = 3000;

// Special case: Old Zalo versions return timestamps with 12-hour format
// instead of 24-hour. We normalize here.
let normalized_time = normalize_timestamp(&raw_time);

// HACK: Zalo Web sometimes doesn't trigger MutationObserver for edited messages.
// Workaround: Fetch full chat history every 5 minutes.
if should_full_refresh { ... }
```

**✗ Bad comments:**

```rust
// Don't comment the obvious
let is_online = true;  // ✓ Don't need comment

// Set is_online to true
is_online = true;      // ✗ Bad: restates code

// Get all messages
let messages = db.query_all();  // ✗ Code already says this
```

### Documentation Comments

```rust
/// Reads new messages from Zalo Desktop via AX API.
///
/// Polls every 3 seconds, returns only unread messages.
/// Automatically handles multi-line messages.
///
/// # Arguments
/// * `max_messages` - Limit number of messages per poll (default: 100)
///
/// # Returns
/// Vector of new messages in chronological order
///
/// # Errors
/// Returns `PollingError::ZaloNotFound` if Zalo process not running
/// Returns `PollingError::AXPermissionDenied` if accessibility disabled
pub fn poll_messages(max_messages: usize) -> Result<Vec<Message>, PollingError> { ... }
```

```typescript
/**
 * Generates AI-powered draft reply using Groq API.
 *
 * Falls back to template matching if input is anonymous.
 * Always anonymizes message content sent to API.
 *
 * @param conversation - Full conversation history
 * @param tone - Desired tone: 'professional' | 'casual' | 'friendly'
 * @returns Generated draft text or template match
 * @throws {DraftError} If API fails and no template matches
 */
export async function generateDraft(
  conversation: Message[],
  tone: 'professional' | 'casual' | 'friendly'
): Promise<string> { ... }
```

## Testing

### Test Organization

```
// Rust: Co-locate tests with code
agent/src/message_parser.rs
// ... implementation
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_message() {
        let raw = "Đồng ý, em sẽ gửi hôm nay";
        let msg = parse_message(raw).unwrap();
        assert_eq!(msg.content, "Đồng ý, em sẽ gửi hôm nay");
    }
}
```

```
// TypeScript: Co-locate tests
src/lib/api/client.ts
src/lib/api/client.test.ts
```

### Test Naming

```rust
#[test]
fn parse_message_with_emoji() { ... }

#[test]
fn send_message_respects_rate_limit() { ... }

#[test]
fn rate_limit_resets_after_one_hour() { ... }

#[test]
fn ai_draft_returns_error_if_groq_api_fails() { ... }
```

## Code Review Checklist

Before submitting PR, ensure:

- [ ] All files follow naming conventions
- [ ] No files exceed 200 LOC (split if needed)
- [ ] All errors are handled explicitly (no silent failures)
- [ ] All public functions have documentation
- [ ] Type safety: no `any`, `string` where possible
- [ ] Constants use UPPER_SNAKE_CASE
- [ ] Variables use camelCase (TS) or snake_case (Rust)
- [ ] Boolean variables prefixed with `is`, `has`, `should`, `can`
- [ ] Comments explain WHY, not WHAT
- [ ] No `console.log` in production code (use logger)
- [ ] Tests pass: `pnpm turbo test`
- [ ] Linting passes: `pnpm turbo lint`
- [ ] Code follows patterns in existing files

## Logging Standards

### Rust

```rust
use log::{info, warn, error, debug};

// Info: Important milestones
info!("Agent started, listening on http://localhost:3000");
info!("Polled {} new messages from Zalo", messages.len());

// Warn: Recoverable issues
warn!("Groq API rate limited, falling back to templates");
warn!("Message send failed, will retry (attempt 2/3)");

// Error: Unrecoverable issues that need user attention
error!("Failed to connect to SQLite database: {}", err);
error!("Zalo account may be banned (health score: 5/100)");

// Debug: Development diagnostics
debug!("Parsed message: {:?}", msg);
debug!("Rate limit check: {}/{} messages/minute", current, limit);
```

### TypeScript

```typescript
import { logger } from '$lib/logger';

// Info
logger.info('Web UI connected to agent', { agentVersion: '1.0.0' });

// Warn
logger.warn('Draft generation slow', { durationMs: 15000 });

// Error
logger.error('Failed to send message', { error: err, messageId: msg.id });

// Debug
logger.debug('API call started', { endpoint: '/messages', method: 'POST' });
```

### No `console.log` in Production

```typescript
// ✗ Bad
console.log('Message sent');
console.error('Error:', error);

// ✓ Good
logger.info('Message sent');
logger.error('Error occurred', { error });
```

## Performance Guidelines

### When to Optimize

1. **Measure first** — Use profiler before optimizing
2. **Big-O matters** — O(n²) loop is worth fixing
3. **User-facing latency** — <100ms for UI, <10s for API
4. **Memory** — <500MB for agent, <100MB for web

### When NOT to Optimize

1. **Code clarity** — Choose readable over clever
2. **Premature optimization** — Don't optimize paths with <1% execution time
3. **Complex algorithms** — Unless proven bottleneck
4. **Early stage** — Validate idea before micro-optimizing

## Svelte-Specific Standards

### Reactive Statements

```svelte
<!-- ✓ Good: Clear reactivity -->
<script>
  let count = 0;

  // Re-run when count changes
  $: doubledCount = count * 2;

  // Re-run side effects
  $: if (count > 10) console.log('Count exceeded 10');
</script>

<!-- ✗ Bad: Implicit dependencies -->
<script>
  let count = 0;
  let doubledCount;  // Not reactive
  doubledCount = count * 2;  // Only runs once
</script>
```

### Store Usage

```svelte
<!-- ✓ Good: Use auto-subscription with $ -->
<script>
  import { appStore } from '$lib/stores/app';
  // $appStore automatically unsubscribes on destroy
</script>

<h1>{$appStore.title}</h1>

<!-- ✗ Bad: Manual subscription -->
<script>
  let title;
  appStore.subscribe(store => title = store.title);
  // Must manually unsubscribe
</script>
```

## Database Standards (SQLite)

### Column Naming

```sql
-- ✓ Good: snake_case, descriptive
CREATE TABLE messages (
  id TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL,
  sender_id TEXT NOT NULL,
  content TEXT NOT NULL,
  sent_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  is_delivered BOOLEAN DEFAULT FALSE
);

-- ✗ Bad
CREATE TABLE messages (
  id TEXT,
  conv_id TEXT,        -- Abbreviation
  senderID TEXT,       -- camelCase
  msg TEXT,            -- Vague
  ts TIMESTAMP         -- Abbreviation
);
```

### Migrations

```sql
-- migrations/001_create_messages_table.sql
-- Always use migration files, never raw SQL
-- File name format: NNN_description.sql

CREATE TABLE IF NOT EXISTS messages (
  id TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL,
  sender_id TEXT NOT NULL,
  content TEXT NOT NULL,
  sent_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (conversation_id) REFERENCES conversations(id),
  FOREIGN KEY (sender_id) REFERENCES contacts(id)
);

CREATE INDEX idx_messages_conversation_id ON messages(conversation_id);
CREATE INDEX idx_messages_sent_at ON messages(sent_at);
```

## Configuration Management

### Environment Variables

```bash
# .env.example — commit this
VITE_PORT=3333
GROQ_API_KEY=your-groq-key
ZALO_OA_ACCESS_TOKEN=your-token

# .env.local — never commit this
GROQ_API_KEY=sk-...actual-secret...
```

### Typed Configuration (Rust)

```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub groq_api_key: String,
    pub polling_interval_ms: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Config {
            port: env::var("VITE_PORT")
                .unwrap_or("3333".to_string())
                .parse()?,
            groq_api_key: env::var("GROQ_API_KEY")
                .map_err(|_| ConfigError::MissingKey("GROQ_API_KEY"))?,
            polling_interval_ms: 3000,
        })
    }
}
```

## Summary Checklist

Every commit should:
- ✓ Follow naming conventions (snake_case, camelCase, UPPER_SNAKE_CASE)
- ✓ Keep files under 200 LOC
- ✓ Handle errors explicitly (no panics, no `any`)
- ✓ Add documentation for public items
- ✓ Add tests for new logic
- ✓ Use logger instead of `console.log`
- ✓ Comment WHY, not WHAT
- ✓ Pass linting and tests

---

**For specific component standards, see:**
- Rust: `agent/src/` examples
- Svelte: `apps/web/src/` examples
- Tests: Existing `.test.ts` and `#[cfg(test)]` examples
