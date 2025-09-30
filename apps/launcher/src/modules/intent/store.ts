import { create } from 'zustand';
import type { ParsedPlanSummary, ExecuteResponse, HistoryItem } from '@suite/ui';
import { parseIntent, dryRun, executePlan, listHistory } from './api';

interface IntentState {
  input: string;
  plan?: ParsedPlanSummary;
  executing: boolean;
  lastRun?: ExecuteResponse;
  history: HistoryItem[];
  historyCursor?: number | null;
  loadingHistory: boolean;
  error?: string;
  setInput(v: string): void;
  parse(explain?: boolean): Promise<void>;
  run(dryRunFlag: boolean): Promise<void>;
  loadMoreHistory(): Promise<void>;
  refreshHistory(): Promise<void>;
}

export const useIntentStore = create<IntentState>((set, get: () => IntentState) => ({
  input: '',
  executing: false,
  history: [],
  loadingHistory: false,
  setInput: (v: string) => set({ input: v }),
  parse: async (explain?: boolean) => {
    const input = get().input.trim();
    if (!input) return;
    try {
      const plan = await parseIntent({ input, explain });
      set({ plan, error: undefined });
    } catch (e: any) {
      set({ error: e?.message || 'parse failed' });
    }
  },
  run: async (dryRunFlag: boolean) => {
    const { plan, input } = get();
    if (!plan && !input.trim()) return;
    set({ executing: true });
    try {
      const resp = dryRunFlag ? await dryRun({ input }) : await executePlan({ input });
      set({ lastRun: resp, error: undefined });
      // refresh history after run
      await get().refreshHistory();
    } catch (e: any) {
      set({ error: e?.message || 'execute failed' });
    } finally {
      set({ executing: false });
    }
  },
  refreshHistory: async () => {
    set({ loadingHistory: true });
    try {
      const h = await listHistory({ limit: 20 });
      set({ history: h.items, historyCursor: h.nextAfter ?? null });
    } catch (e) {
      // ignore
    } finally {
      set({ loadingHistory: false });
    }
  },
  loadMoreHistory: async () => {
    const { historyCursor, history } = get();
    if (!historyCursor) return;
    set({ loadingHistory: true });
    try {
      const h = await listHistory({ limit: 20, after: historyCursor });
      set({
        history: [...history, ...h.items],
        historyCursor: h.nextAfter ?? null,
      });
    } finally {
      set({ loadingHistory: false });
    }
  },
}));
