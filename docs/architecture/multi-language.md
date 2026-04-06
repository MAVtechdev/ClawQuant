# Multi-language architecture

ClawQuant uses a **multi-language architecture**:

- **TypeScript** — Live agent orchestration, tool routing, connectors (Telegram, Web, MCP), exchange/broker adapters, config, and the web UI. Today this is the `src/` + `ui/` tree and `packages/schemas-ts` for shared artifacts.
- **Python** — Research brain: backtests, walk-forward framing, trade labeling, attribution hooks, benchmarks, and experiment reports (`python/claw_quant/`).
- **Rust** — Performance layer: technical indicators, candle replay, and room for tick/portfolio kernels (`rust/`).

## Target layout (migration)

The production runtime remains under `src/` and `ui/` while the repo grows into:

```text
apps/runtime-ts     ← maps from today's src/ (incremental)
apps/web-ts         ← maps from today's ui/
python/claw_quant   ← research + learning
rust/*              ← indicator-engine, replay-core, …
packages/schemas-ts ← JSON/Zod contracts (e.g. trade learning records)
```

This avoids a big-bang directory move while still presenting a credible multi-language story on GitHub.

## Integration surface

- **Files under `data/`** — Sessions, configs, future `learning/trades/*.json` (schema in `packages/schemas-ts`).
- **No shared in-process FFI yet** — Python and Rust are invoked as separate CLIs or services; the TypeScript agent remains the source of truth for live execution until you add bridges (e.g. subprocess, HTTP, or protobuf).
