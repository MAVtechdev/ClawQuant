# Scheduling Subsystem

ClawQuant's scheduling subsystem: heartbeat, cron, and delivery in a clean-room design (dual-trigger pattern common to local agent runtimes).

## Design Philosophy

1. **Scheduler is the single alarm clock** — All events that need to wake the agent (heartbeat, cron, manual trigger) converge through `scheduler.requestWake()`
2. **Cron never calls the agent directly** — Cron injects events into the system event queue, then wakes the scheduler; the agent sees these events in its normal conversation context
3. **Agent self-decides whether to deliver** — Via an ack token (default `HEARTBEAT_OK`), the agent can suppress delivery when there's nothing to report
4. **File-driven** — Cron jobs persist as JSON, delivery queue persists as individual files, recoverable after crash
5. **Module isolation** — 5 files each own their domain, glued together in `main.ts`, no cross-imports (except cron→agent-events, scheduler→agent-events)

## File Inventory

```
src/core/
  agent-events.ts       Event bus + system event queue
  scheduler.ts          Heartbeat scheduling + wake coalescing + active hours + ack token
  cron.ts               File-persisted cron job engine
  delivery.ts           Outbound message queue + retry + crash recovery
  connector-registry.ts Connector registry + "last interaction" delivery routing
```

## Architecture Overview

```
            ┌──────────────┐
            │  Cron Engine │
            │  (scheduled) │
            └──────┬───────┘
                   │ enqueueSystemEvent()
                   │ requestWake('cron')
                   ▼
┌──────────┐   ┌──────────────┐   ┌───────────────┐
│ Manual / │──▶│  Scheduler   │◀──│   Connector   │
│ Hook     │   │ (alarm+merge)│   │ touchInteraction()
└──────────┘   └──────┬───────┘   └───────────────┘
                      │
                      │ runOnce()
                      ▼
               ┌──────────────┐
               │    Engine     │
               │ askWithSession│
               │ (heartbeat    │
               │  session)     │
               └──────┬───────┘
                      │
                      │ stripAckToken()
                      ▼
              ┌───────────────┐
              │ Has content   │
              │ to deliver?   │
              │ shouldSkip?   │
              └───┬───────┬───┘
                  │no     │yes
                  ▼       ▼
           ┌──────────┐  (silent)
           │ Delivery  │
           │ enqueue() │
           │ deliver() │
           │ ack()     │
           └──────────┘
```

## Module Details

### 1. Agent Events (`agent-events.ts`)

In-memory event bus + system event queue — two independent features in one file.

**Event Bus** — Purely observational, does not affect control flow:
```typescript
emit('heartbeat', { status: 'sent', reason: 'interval', durationMs: 1234 })
emit('delivery', { status: 'failed', channel: 'telegram', error: '...' })

// Subscribe
const unsub = on((event) => console.log(event))
const unsub2 = onStream('heartbeat', (event) => { ... })
```

Each stream has an independent monotonically increasing sequence number for ordering and deduplication.

**System Event Queue** — Communication channel from cron to agent:
```typescript
// Cron injects
enqueueSystemEvent({
  id: 'cron:abc:1234567890',
  source: 'cron',
  text: 'Check BTC RSI',
  contextKey: 'cron:abc',  // Events with the same key are deduplicated (newer replaces older)
})

// Scheduler drains on flush
const events = drainSystemEvents()  // Retrieves and clears
```

Queue limit is 50 items; oldest are evicted when exceeded. No persistence — pending system events are lost on crash, but cron will re-trigger them.

### 2. Scheduler (`scheduler.ts`)

Core scheduler, responsibilities:
- Wake the agent at intervals (e.g. 30m)
- Coalesce multiple wake requests within a short window (250ms)
- Enforce active hours checks
- Call the `runOnce` callback

**Creation:**
```typescript
const scheduler = createScheduler(
  { heartbeat: config.scheduler.heartbeat },
  runOnce,  // (opts) => Promise<HeartbeatResult>
)
```

**Wake Coalescing:**

Multiple `requestWake()` calls within 250ms trigger only one execution, using the highest-priority reason:

| Reason     | Priority | Source                          |
|------------|----------|---------------------------------|
| `retry`    | 0        | Auto-retry after failed execution |
| `interval` | 1        | Heartbeat timer                 |
| `cron`     | 2        | Cron job trigger                |
| `message`  | 2        | User message (reserved)         |
| `manual`   | 3        | Manual trigger                  |
| `hook`     | 3        | Webhook / external integration  |

Example:
```
t=0ms   requestWake('interval')   → pending = interval (pri 1)
t=50ms  requestWake('cron')       → pending = cron (pri 2, upgraded)
t=250ms flush()                   → executes once, reason='cron'
```

**Active Hours:**

Only applies to `interval` heartbeats. Cron, manual, and hook are unrestricted.

```json
{
  "activeHours": {
    "start": "09:00",
    "end": "22:00",
    "timezone": "Asia/Shanghai"
  }
}
```

Supports crossing midnight (e.g. `"22:00"` → `"06:00"`), IANA timezones, and `"local"` for the system timezone.

**Ack Token Self-Decision:**

After the agent replies, `stripAckToken()` checks:
1. Empty reply → `ok-empty`, no delivery
2. Reply contains ack token and remaining text ≤ `ackMaxChars` → `ok-ack`, no delivery
3. Substantive content → `sent`, proceed with delivery

Default ack token is `HEARTBEAT_OK`, default `ackMaxChars` is 300. The agent's prompt tells it: reply `HEARTBEAT_OK` when there's nothing to report.

**Execution Mutex:**

A `running` flag ensures only one `runOnce` executes at a time. New wake requests during execution are queued until the current run completes.

**Auto-Retry on Failure:**

When `runOnce` throws, a `requestWake('retry')` is automatically issued after 1 second (lowest priority, won't preempt normal scheduling).

### 3. Cron (`cron.ts`)

File-driven scheduled task engine.

**Three Schedule Types:**

| Kind    | Format                   | Description                        |
|---------|--------------------------|------------------------------------|
| `at`    | ISO timestamp            | One-shot, auto-disabled after execution |
| `every` | Duration `"2h"`, `"30m"` | Periodic execution                 |
| `cron`  | 5-field cron expression  | Standard cron                      |

**Two Session Targets:**

| Target     | Behavior                                                          |
|------------|-------------------------------------------------------------------|
| `main`     | Injects system event + wakes scheduler, agent processes in main context |
| `isolated` | (Reserved) Runs in a separate session, doesn't affect main conversation |

**Persistence:**

All jobs are stored in a single JSON file (default `data/cron/jobs.json`), written atomically (tmp + rename).

**Error Backoff:**

On consecutive failures, the next run time is delayed:
```
1st failure → +30s
2nd failure → +1m
3rd failure → +5m
4th failure → +15m
5th+ failure → +60m
```

**Timer Drift Protection:**

`setTimeout` is imprecise for large values, so timers are recalculated every 60 seconds at most.

**Main Session Trigger Flow:**
```
cron timer fires
  → executeJob(job)
    → enqueueSystemEvent({ source: 'cron', text: job.payload, contextKey: `cron:${job.id}` })
    → onWake('cron')  // = scheduler.requestWake('cron')
  → save()  // Update nextRunAtMs
  → armTimer()  // Set next timer
```

### 4. Delivery (`delivery.ts`)

File-driven outbound message queue.

**Lifecycle:**
```
enqueue()  →  Write file to data/delivery-queue/{uuid}.json
deliver()  →  Send via connector
ack()      →  Delete file
fail()     →  Update file (retryCount++, lastError)
moveToFailed()  →  Move to data/delivery-queue/failed/ subdirectory
```

**Retry Backoff:**
```
1st retry → 5s
2nd retry → 25s
3rd retry → 2m
4th retry → 10m
Exceeds maxRetries (default 5) → moved to failed/
```

**Crash Recovery:**

On startup, `recoverPending()` scans the queue directory, sorts by `enqueuedAt`, and retries each item. Has a time budget (default 60s); remaining items are deferred if the budget is exceeded.

**Usage in `runOnce`:**
```typescript
// Write to disk first (prevent loss)
const entryId = await enqueue(config, { channel, to, text })

// Then attempt immediate delivery
try {
  await target.deliver(text)
  await ack(config, entryId)      // Success → delete file
} catch {
  // Failure → file remains, recoverPending() will retry on next startup
}
```

### 5. Connector Registry (`connector-registry.ts`)

Connector registry that solves "who to deliver to".

**Registration (when Telegram plugin starts):**
```typescript
const unregister = registerConnector({
  channel: 'telegram',
  to: String(chatId),
  deliver: async (text) => { client.sendMessage({ chatId, text }) },
})
```

**Recording Interactions (each time the user sends a message):**
```typescript
touchInteraction('telegram', String(message.chatId))
```

**Resolving Delivery Target:**
```typescript
const target = resolveDeliveryTarget()
// → Returns the connector from the last touchInteraction
// → If no interaction recorded, falls back to the first registered connector
// → If no connectors registered, returns null
```

Design intent: single-tenant, multi-channel. One user may be reachable via Telegram, Discord, Webhooks, etc. Delivery always goes to the channel of last interaction. Adding new connectors in the future only requires `registerConnector()` + `touchInteraction()` — no changes to scheduling logic.

## Configuration

`data/config/scheduler.json` (all fields have defaults, works even if the file doesn't exist):

```json
{
  "heartbeat": {
    "enabled": true,
    "every": "30m",
    "prompt": "Read HEARTBEAT.md and check if anything needs attention. Reply HEARTBEAT_OK if nothing to report.",
    "ackToken": "HEARTBEAT_OK",
    "ackMaxChars": 300,
    "activeHours": {
      "start": "09:00",
      "end": "22:00",
      "timezone": "Asia/Shanghai"
    }
  },
  "cron": {
    "enabled": true,
    "storePath": "data/cron/jobs.json"
  },
  "delivery": {
    "queueDir": "data/delivery-queue",
    "maxRetries": 5
  }
}
```

Minimal configuration to enable:
```json
{
  "heartbeat": { "enabled": true }
}
```

## Startup Sequence

```
main()
  1. loadConfig()
  2. Initialize trading / wallet / sandbox / brain
  3. new Engine(...)
  4. engine.use(TelegramPlugin)          ← Register plugin (not started yet)
  5. new SessionStore('heartbeat')       ← Heartbeat-dedicated session
  6. createScheduler(config, runOnce)    ← Heartbeat timer starts
  7. createCronEngine(config, onWake)    ← Created but not started
  8. engine.start()
     8a. TelegramPlugin.start()          ← Connect API, registerConnector()
     8b. engine.onReady()
         8b-i.  cronEngine.start()       ← Load jobs.json, arm timer
         8b-ii. recoverPending()         ← Scan queue, retry failed deliveries
     8c. while (!stopped) tick()         ← Enter main loop
```

`onReady` guarantees: cron and delivery recovery start only after connectors are registered.

## Data Flow

### Heartbeat Flow
```
setInterval(30m)
  → requestWake('interval')
  → [250ms coalesce]
  → flush()
    → activeHours check (interval only)
    → drainSystemEvents()
    → runOnce({ reason, prompt, systemEvents })
      → engine.askWithSession(prompt, heartbeatSession)
      → stripAckToken(response)
      → if shouldSkip: return ok-ack
      → resolveDeliveryTarget()
      → enqueue() → deliver() → ack()
```

### Cron Flow
```
cron timer fires
  → enqueueSystemEvent({ source: 'cron', text: payload })
  → scheduler.requestWake('cron')
  → [250ms coalesce — may merge with heartbeat]
  → flush()
    → drainSystemEvents() → retrieve cron events
    → runOnce({ systemEvents: [{source:'cron', text:'...'}] })
      → prompt assembly: heartbeat prompt + system events
      → agent sees full message, responds normally
      → strip + deliver (same as heartbeat)
```

### User Message Flow
```
Telegram message arrives
  → touchInteraction('telegram', chatId)    ← Record last interaction
  → handleMessage() → engine.askWithSession()   ← Normal chat
  → sendReply()                                  ← Direct reply

  (Next heartbeat/cron delivery will go to this chatId)
```

## Disk Structure

```
data/
  config/
    scheduler.json          ← Scheduling configuration
  sessions/
    heartbeat.jsonl         ← Heartbeat-dedicated session (JSONL format)
  cron/
    jobs.json               ← Cron job list
  delivery-queue/
    {uuid}.json             ← Pending delivery entries
    failed/
      {uuid}.json           ← Entries that exceeded retry limit
```

## Tests

All modules have corresponding `.spec.ts` files, totaling 83 test cases:

| File                        | Tests | Key Technique             |
|-----------------------------|-------|---------------------------|
| `agent-events.spec.ts`      | 14    | In-memory state, `_resetForTest()` |
| `scheduler.spec.ts`         | 26    | `vi.useFakeTimers()`      |
| `cron.spec.ts`              | 15    | `mkdtemp` + temp directory |
| `delivery.spec.ts`          | 17    | `mkdtemp` + temp directory |
| `connector-registry.spec.ts`| 11    | In-memory state, `_resetForTest()` |

Run:
```bash
npx vitest run src/core/*.spec.ts
```

## Design Decision Records

### Why not BullMQ?
ClawQuant is a single-process application and doesn't need a Redis-backed distributed task queue. Agent Events (84-line EventEmitter) + file persistence already meets the requirements.

### Why do heartbeat and cron share the scheduler?
Here, two independent trigger lines (cron timer + heartbeat interval) converge on the same execution entry point. Sharing the scheduler naturally supports wake coalescing — cron and heartbeat within the same 250ms window execute only once.

### Why does cron inject via system events instead of calling the agent directly?
To maintain a unified entry point for the agent. The agent doesn't need to know whether a prompt came from a heartbeat or cron — it just sees a message and processes it normally. This makes the scheduling system completely transparent to the agent.

### Why does delivery write to disk before sending?
To prevent loss. The `enqueue()` → `deliver()` → `ack()` three-step operation ensures no message is lost if any step fails. Worst case: delivery succeeds but the process crashes before `ack()` — on next startup it will re-send once (at-least-once semantics).

### Why use a "last interaction" strategy for delivery routing?
Delivery routing is connector-first: in a single-tenant scenario, wherever the user last talked to the bot, the bot replies in that channel. Adding new connectors only requires `registerConnector()` + `touchInteraction()` — no changes to scheduling logic.
