/**
 * Stub: channels/plugins/types.ts
 *
 * The original re-exports from types.adapters.ts, types.core.ts, and
 * types.plugin.ts — each pulling in deep deps. The browser subsystem only
 * needs: ChannelMessageActionName (runtime), ChannelId, ChannelMeta (types).
 *
 * Upstream: openclaw/src/channels/plugins/types.ts
 */

// Runtime re-export — original
export { CHANNEL_MESSAGE_ACTION_NAMES } from "./message-action-names.js";

// Type re-exports — only the ones actually used by the browser chain
export type ChannelMessageActionName = import("./message-action-names.js").ChannelMessageActionName;

export type ChannelId = string;

export type ChannelMeta = {
  id: string;
  label: string;
  selectionLabel?: string;
  detailLabel?: string;
  docsPath: string;
  docsLabel?: string;
  blurb: string;
  systemImage?: string;
  selectionDocsPrefix?: string;
  selectionDocsOmitLabel?: boolean;
  selectionExtras?: string[];
  aliases?: string[];
};

export type ChannelPlugin = {
  id: ChannelId;
  meta: ChannelMeta;
  [key: string]: any;
};
