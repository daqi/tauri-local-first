export interface ParsedPlanSummary {
  planId: string;
  strategy: string;
  batches: number;
  conflicts: number;
  cacheHit: boolean;
  signature?: string | null;
  explain?: any;
}

export interface ExecuteActionResult {
  intentId: string;
  status: string;
  reason?: string | null;
  retryHint?: string | null;
  durationMs?: number | null;
  predictedEffects?: any;
}

export interface ExecuteResponse {
  planId: string;
  overallStatus: string;
  actions: ExecuteActionResult[];
  batches: number;
  conflicts: number;
  cacheHit: boolean;
}

export interface HistoryItem {
  signature: string;
  input: string;
  overallStatus: string;
  planSize: number;
  explainUsed: boolean;
  createdAt: number;
  intents: string[];
}

export interface HistoryListResponse {
  items: HistoryItem[];
  nextAfter?: number | null;
}
