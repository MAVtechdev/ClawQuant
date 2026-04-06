/**
 * Stub: config/sessions.ts
 * Upstream: openclaw/src/config/sessions.ts
 * Only the SessionEntry type is used (by gateway/session-utils.types.ts).
 */
export type SessionEntry = {
  key: string;
  label?: string;
  agentId?: string;
  model?: string;
  thinking?: string;
  spawnDepth?: number;
  [key: string]: unknown;
};
