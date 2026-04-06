/**
 * Stub: config/types.ts
 *
 * The original barrel re-exports ~30 types.*.ts files. We only re-export the
 * ones the browser subsystem actually references.
 *
 * Upstream: openclaw/src/config/types.ts
 */

export * from "./types.browser.js";
export * from "./types.gateway.js";

// ClawBrowserRootConfig — the root config type. In the original codebase this is a
// massive Zod-validated type built from all types.*.ts files. We provide a
// permissive type alias so the browser subsystem can access arbitrary config
// paths without pulling in the full schema.
export type ClawBrowserRootConfig = Record<string, any>;
