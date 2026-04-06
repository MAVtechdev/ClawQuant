/**
 * Stub: utils/delivery-context.ts
 * Upstream: openclaw/src/utils/delivery-context.ts
 * Only DeliveryContext type is used (by gateway/session-utils.types.ts).
 */
export type DeliveryContextSessionSource = {
  agentId?: string;
  channel?: string;
  sessionKey?: string;
};

export type DeliveryContext = {
  channel?: string;
  to?: string;
  threadId?: string | number;
  groupId?: string | null;
  groupChannel?: string | null;
  groupSpace?: string | null;
  source?: DeliveryContextSessionSource;
  [key: string]: unknown;
};
