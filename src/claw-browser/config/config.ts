/**
 * Stub: config/config.ts
 *
 * Replaces the original barrel re-export that pulls in the massive config IO
 * pipeline (~20+ files). We only re-export what the browser subsystem actually
 * uses at runtime.
 *
 * Upstream: openclaw/src/config/config.ts
 */

// --- Re-exports from paths.ts (original) ---
export {
  resolveGatewayPort,
  resolveConfigPath,
  STATE_DIR,
  resolveStateDir,
  resolveOAuthDir,
} from "./paths.js";

// --- Re-exports from port-defaults.ts (original) ---
export {
  deriveDefaultBrowserCdpPortRange,
  deriveDefaultBrowserControlPort,
  DEFAULT_BROWSER_CONTROL_PORT,
} from "./port-defaults.js";

// --- Type re-exports ---
export type { ClawBrowserRootConfig } from "./types.js";
export type { BrowserConfig, BrowserProfileConfig } from "./types.browser.js";
// Re-export ALL gateway types — gateway/*.ts imports many of them through this barrel
export type {
  GatewayAuthConfig,
  GatewayAuthMode,
  GatewayAuthRateLimitConfig,
  GatewayBindMode,
  GatewayTailscaleConfig,
  GatewayTailscaleMode,
  GatewayTlsConfig,
  GatewayTrustedProxyConfig,
} from "./types.gateway.js";

// --- loadConfig stub ---
// The original reads YAML/JSON through a deep validation pipeline.
// We provide a minimal stub that reads a JSON config file directly.
import { readFileSync, existsSync } from "node:fs";
import { resolve } from "node:path";
import { resolveStateDir } from "./paths.js";
import type { ClawBrowserRootConfig } from "./types.js";

let _cachedConfig: ClawBrowserRootConfig | null = null;

export function loadConfig(): ClawBrowserRootConfig {
  if (_cachedConfig) return _cachedConfig;
  const configPath = resolve(resolveStateDir(), "config.json");
  if (!existsSync(configPath)) {
    // Standalone mode: no Gateway, no node proxy.
    // Disable node browser proxy so the browser tool doesn't attempt
    // to connect ws://127.0.0.1:18789 (the OpenClaw Gateway).
    _cachedConfig = {
      gateway: { nodes: { browser: { mode: "off" } } },
    } as ClawBrowserRootConfig;
    return _cachedConfig;
  }
  try {
    _cachedConfig = JSON.parse(readFileSync(configPath, "utf-8")) as ClawBrowserRootConfig;
  } catch {
    _cachedConfig = {
      gateway: { nodes: { browser: { mode: "off" } } },
    } as ClawBrowserRootConfig;
  }
  return _cachedConfig;
}

export function writeConfigFile(_config: ClawBrowserRootConfig): void {
  // Stub — implement when config persistence is needed
}

export function createConfigIO() {
  return { load: loadConfig, loadConfig, save: writeConfigFile };
}
