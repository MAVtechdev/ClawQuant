/**
 * Stub: agents/sandbox/fs-bridge.ts
 * Upstream: openclaw/src/agents/sandbox/fs-bridge.ts
 * Only the SandboxFsBridge type is used (by agents/sandbox/types.ts).
 */
export type SandboxFsBridge = {
  read: (path: string) => Promise<Buffer>;
  write: (path: string, content: Buffer) => Promise<void>;
  stat: (path: string) => Promise<{ size: number; mtime: Date } | null>;
  list: (dir: string) => Promise<string[]>;
  [key: string]: unknown;
};
