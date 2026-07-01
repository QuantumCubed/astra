---
created: 2026-07-01
updated: 2026-07-01
status: planned
---

[[INTEGRATIONS|← Integrations]]

# Discord Integration

## Goal

Add a Discord bot that lets authorized users (those with a specific server role) chat with Astra from Discord, and lets the LLM manage the server via tools.

---

## Library Choice

**serenity** (`tokio-tungstenite` is the underlying WebSocket transport — consistent with the architecture commitment).

Handles gateway complexity that would otherwise need to be reimplemented: heartbeat + jitter, IDENTIFY/RESUME opcodes, session invalidation, reconnect backoff, zlib-compressed payloads, intent bitmasks.

---

## Config Keys (`.astra/astra.conf`)

```
DISCORD_BOT_TOKEN=...
DISCORD_GUILD_ID=...             # u64 server ID
DISCORD_AUTHORIZED_ROLE_ID=...   # u64 role ID users must have
```

If `DISCORD_BOT_TOKEN` is absent, Astra starts normally without Discord (same graceful degradation pattern as `ha_client`).

---

## Architecture

### New: `DiscordState`

Stored as `Option<Arc<DiscordState>>` on `AppState` (same pattern as `ha_client`).

```
DiscordState {
    guild_id: u64,
    authorized_role_id: u64,
    http: Arc<serenity::http::Http>,     // REST calls for tools
    conversations: Arc<Mutex<HashMap<String, Conversation>>>,  // keyed by Discord user ID
}
```

Per-user conversation context — each authorized Discord user gets their own sliding-window `Conversation`. Populated lazily on first message from a user.

### Module Structure

```
src/integrations/discord.rs                — add mod event_handler; mod agent;
src/integrations/discord/
    discord_connection.rs (expand stub)    — DiscordState, start_discord_bot()
    event_handler.rs                       — serenity EventHandler impl
    agent.rs                               — run_discord_agent_loop()
```

### Message Flow

```
Discord gateway (serenity)
    → AstraDiscordHandler::message()
        → role check (skip if unauthorized)
        → look up / create Conversation for this user
        → post "..." placeholder message to channel
        → run_discord_agent_loop()
            loop:
                client::chat() → collect NDJSON stream → (full_content, tool_calls)
                if tool_calls → dispatch_tool() → add_tool_result() → loop
                else → break
            edit placeholder with collected text
            (split if > 2000 chars — Discord's message limit)
```

### Agent Loop Design

`run_discord_agent_loop` does **not** stream tokens to Discord. It collects the full Ollama response, dispatches any tool calls, then edits the placeholder message with the final text.

The existing `run_agent_loop` in `handlers/ws.rs` is **unchanged** — it continues to stream to the WebSocket client with TTS. Discord gets its own loop because the output behavior is fundamentally different (no streaming, no TTS, message editing model).

Shared infrastructure reused as-is: `client::chat()`, `dispatch::dispatch_tool()`, `Conversation`.

---

## Tools (Initial Scope)

### Messaging
- `discord_send_message(channel_id, content)` — send a new message
- `discord_edit_message(channel_id, message_id, content)` — edit an existing message
- `discord_delete_message(channel_id, message_id)` — delete a message

### Role Management
- `discord_assign_role(user_id, role_id)` — add a role to a user
- `discord_remove_role(user_id, role_id)` — remove a role from a user
- `discord_list_members_with_role(role_id)` — list users who have a given role

All tool implementations take `&serenity::http::Http` (from `DiscordState`) and go through the standard `registry → dispatch → implementations` pipeline.

---

## Files Changed

| File | Change |
|---|---|
| `Cargo.toml` | Add `serenity` (verify version on crates.io before pinning) |
| `src/backend/config.rs` | Add `discord_bot_token()`, `discord_guild_id()`, `discord_authorized_role_id()` |
| `src/backend/state.rs` | Add `discord: Option<Arc<DiscordState>>` |
| `src/main.rs` | After `AppState::new()`, conditionally `tokio::spawn(start_discord_bot(...))` |
| `src/integrations/discord.rs` | Add `pub mod event_handler; pub mod agent;` |
| `src/integrations/discord/discord_connection.rs` | Expand stub: `DiscordState`, `start_discord_bot()` |
| `src/integrations/discord/event_handler.rs` | **New** — serenity `EventHandler` impl |
| `src/integrations/discord/agent.rs` | **New** — `run_discord_agent_loop()` |
| `src/tools/registry.rs` | 6 new tool entries |
| `src/tools/dispatch.rs` | 6 new match arms |
| `src/tools/implementations.rs` | 6 new async functions |

---

## Verification

1. `cargo check` clean after adding serenity
2. Start without Discord config → server starts normally
3. Start with Discord config → bot connects, logs show READY
4. Message from unauthorized user → no response
5. Message from authorized user → response posted to Discord
6. Multi-turn: second message from same user → Astra remembers context
7. Ask Astra to send a message to a channel → `discord_send_message` fires
8. Ask Astra to assign a role → `discord_assign_role` fires, user gains role
