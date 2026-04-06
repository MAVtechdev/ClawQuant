import { z } from "zod";

/**
 * On-disk trade learning record (aligns with README `data/learning/trades/*.json`).
 * Runtime TS and Python services should treat this as the contract for promotion gates.
 */
export const tradeLearningRecordSchema = z.object({
  id: z.string(),
  symbol: z.string(),
  policyVersion: z.string(),
  openedAt: z.string(),
  closedAt: z.string().optional(),
  marketSnapshot: z.record(z.string(), z.unknown()).optional(),
  featuresAtEntry: z.record(z.string(), z.number()).optional(),
  reasoningSummary: z.string().optional(),
  action: z.record(z.string(), z.unknown()).optional(),
  fills: z.array(z.record(z.string(), z.unknown())).optional(),
  risk: z
    .object({
      stop: z.number().optional(),
      target: z.number().optional(),
      size: z.number().optional(),
    })
    .optional(),
  outcome: z
    .object({
      horizonBars: z.number().optional(),
      simpleReturn: z.number().optional(),
      realizedPnl: z.number().optional(),
    })
    .optional(),
  attribution: z
    .object({
      entryQuality: z.number().optional(),
      exitQuality: z.number().optional(),
      regimeFit: z.number().optional(),
      sizingFit: z.number().optional(),
    })
    .optional(),
  lesson: z.string().optional(),
  promotionStatus: z.enum(["promoted", "rejected", "queued"]),
});

export type TradeLearningRecord = z.infer<typeof tradeLearningRecordSchema>;
