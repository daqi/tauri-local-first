import { invoke } from '@tauri-apps/api/core';
// Types re-exported from @suite/ui (built) - during dev we rely on source type declarations
// During workspace source usage, import types from source path
import type { ParsedPlanSummary, ExecuteResponse, HistoryListResponse } from '@suite/ui';

export interface ParseIntentParams { input: string; explain?: boolean }
export async function parseIntent(params: ParseIntentParams): Promise<ParsedPlanSummary> {
  const resp = await invoke<any>('parse_intent', { req: { input: params.input, explain: !!params.explain } });
  return resp as ParsedPlanSummary;
}

export interface ExecuteParams { input?: string; planId?: string; dryRun?: boolean; timeoutMs?: number }
export async function dryRun(params: ExecuteParams): Promise<ExecuteResponse> {
  const resp = await invoke<any>('dry_run', { req: { input: params.input, planId: params.planId, dryRun: true } });
  return resp as ExecuteResponse;
}
export async function executePlan(params: ExecuteParams): Promise<ExecuteResponse> {
  const resp = await invoke<any>('execute_plan', { req: { input: params.input, planId: params.planId, dryRun: false, timeoutMs: params.timeoutMs } });
  return resp as ExecuteResponse;
}

export interface ListHistoryParams { limit?: number; after?: number }
export async function listHistory(params: ListHistoryParams = {}): Promise<HistoryListResponse> {
  const resp = await invoke<any>('list_history', { req: params });
  return resp as HistoryListResponse;
}
