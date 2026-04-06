/**
 * Stub: sessions/input-provenance.ts
 * Upstream: openclaw/src/sessions/input-provenance.ts
 */
export const INPUT_PROVENANCE_KIND_VALUES = [
  "external_user",
  "inter_session",
  "internal_system",
] as const;

export type InputProvenanceKind = (typeof INPUT_PROVENANCE_KIND_VALUES)[number];
