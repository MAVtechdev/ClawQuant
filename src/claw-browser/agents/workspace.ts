/**
 * Stub: agents/workspace.ts
 * Upstream: openclaw/src/agents/workspace.ts
 *
 * The original is 547 lines with deep deps (workspace-templates, etc.).
 * Only resolveDefaultAgentWorkspaceDir() is imported by agent-scope.ts.
 */
import { resolve } from "node:path";
import { homedir } from "node:os";

export function resolveDefaultAgentWorkspaceDir(_env?: Record<string, string | undefined>): string {
  return resolve(homedir(), ".openclaw", "workspace");
}

export const DEFAULT_AGENT_WORKSPACE_DIR = resolveDefaultAgentWorkspaceDir();
