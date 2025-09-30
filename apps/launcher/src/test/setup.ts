import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => {
  return {
    invoke: vi.fn(async (cmd: string, _payload: any) => {
      if (cmd === 'parse_intent') {
        return {
          planId: 'p1',
          strategy: 'sequential',
          batches: 1,
          conflicts: 0,
          cacheHit: false,
          signature: 'sig-123',
        };
      }
      if (cmd === 'dry_run') {
        return {
          planId: 'p1',
          overallStatus: 'success',
          actions: [
            { intentId: 'i1', status: 'simulated' },
            { intentId: 'i2', status: 'simulated' },
          ],
          batches: 1,
          conflicts: 0,
          cacheHit: false,
        };
      }
      if (cmd === 'execute_plan') {
        return {
          planId: 'p1',
          overallStatus: 'success',
          actions: [
            { intentId: 'i1', status: 'success', durationMs: 12 },
            { intentId: 'i2', status: 'success', durationMs: 15 },
          ],
          batches: 1,
          conflicts: 0,
          cacheHit: false,
        };
      }
      if (cmd === 'list_history') {
        return {
          items: [
            {
              signature: 'sig-123',
              input: 'hosts:switch(dev)',
              overallStatus: 'success',
              planSize: 2,
              explainUsed: false,
              createdAt: Date.now(),
              intents: ['switch', 'switch'],
            },
          ],
          nextAfter: null,
        };
      }
      return {};
    }),
  };
});
