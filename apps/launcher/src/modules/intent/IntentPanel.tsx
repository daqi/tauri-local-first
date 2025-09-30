import React, { useEffect } from 'react';
import { useIntentStore } from './store';
import { ActionList, ExecutionStatus, HistoryList } from '@suite/ui';

export const IntentPanel: React.FC = () => {
  const { input, setInput, parse, run, plan, lastRun, executing, history, refreshHistory, loadMoreHistory, historyCursor } = useIntentStore();
  useEffect(() => { refreshHistory(); }, [refreshHistory]);
  const actions = lastRun?.actions || [];
  return (
    <div className="flex flex-col gap-4 p-4 max-w-3xl mx-auto">
      <div className="flex gap-2 items-start">
        <textarea
          className="flex-1 border rounded p-2 text-sm h-24 focus:outline-none focus:ring"
          placeholder="Enter intent commands..."
          value={input}
          onChange={e => setInput(e.target.value)}
        />
        <div className="flex flex-col gap-2 w-32">
          <button className="px-2 py-1 text-sm border rounded" onClick={() => parse(false)}>Parse</button>
          <button className="px-2 py-1 text-sm border rounded" onClick={() => run(true)} disabled={executing}>Dry Run</button>
          <button className="px-2 py-1 text-sm border rounded bg-green-600 text-white disabled:opacity-50" onClick={() => run(false)} disabled={executing}>Execute</button>
        </div>
      </div>
      <div className="space-y-2">
        <h3 className="font-semibold text-sm">Plan</h3>
        {plan ? (
          <ExecutionStatus
            overallStatus={lastRun?.overallStatus}
            cacheHit={plan.cacheHit}
            conflicts={plan.conflicts}
            batches={plan.batches}
            explainUsed={!!plan.explain}
            signature={plan.signature || undefined}
          />
        ) : <div className="text-xs text-neutral-500">No plan yet</div>}
      </div>
      <div className="space-y-2">
        <h3 className="font-semibold text-sm">Actions</h3>
  <ActionList actions={actions.map((a: any) => ({ intentId: a.intentId, status: a.status, reason: a.reason || undefined, durationMs: a.durationMs || undefined }))} />
      </div>
      <div className="space-y-2">
        <h3 className="font-semibold text-sm flex items-center gap-2">History <button className="text-[10px] border px-2 py-0.5 rounded" onClick={() => refreshHistory()}>↻</button></h3>
        <HistoryList items={history} hasMore={!!historyCursor} onNextPage={() => loadMoreHistory()} />
      </div>
    </div>
  );
};

export default IntentPanel;
