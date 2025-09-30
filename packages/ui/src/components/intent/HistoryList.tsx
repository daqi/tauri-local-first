import React from 'react';
import { cn } from '../../lib/utils';

export interface HistoryRecordItem {
  signature: string;
  input: string;
  overallStatus: string;
  createdAt: number;
  planSize?: number;
  explainUsed?: boolean;
  intents?: string[];
}

export interface HistoryListProps {
  items: HistoryRecordItem[];
  onNextPage?: (cursor: number) => void;
  hasMore?: boolean; // derive by presence of next cursor externally
  className?: string;
  loading?: boolean;
  emptyHint?: string;
}

const statusDot = (s: string) => {
  const base = 'inline-block w-2 h-2 rounded-full mr-1';
  switch (s) {
    case 'success':
      return base + ' bg-green-500';
    case 'failed':
      return base + ' bg-red-500';
    case 'partial':
      return base + ' bg-yellow-500';
    default:
      return base + ' bg-neutral-400';
  }
};

const formatTime = (ms: number) => {
  try {
    const d = new Date(ms);
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  } catch (_) {
    return String(ms);
  }
};

const HistoryList: React.FC<HistoryListProps> = ({
  items,
  onNextPage,
  hasMore,
  className,
  loading,
  emptyHint,
}) => {
  if (loading) return <div className={cn('text-sm text-neutral-500', className)}>Loading…</div>;
  if (!items.length)
    return (
      <div className={cn('text-sm text-neutral-400 italic', className)}>
        {emptyHint || 'No history yet'}
      </div>
    );
  return (
    <div className={cn('flex flex-col gap-2', className)}>
      <ul className="space-y-1 text-xs">
        {items.map((r) => (
          <li
            key={r.signature}
            className="border rounded p-2 bg-white/70 dark:bg-neutral-900/50 border-neutral-200 dark:border-neutral-700"
          >
            <div className="flex items-center justify-between gap-2">
              <div className="flex items-center gap-1 min-w-0">
                <span className={statusDot(r.overallStatus)} />
                <span
                  className="font-medium capitalize truncate max-w-[120px]"
                  title={r.overallStatus}
                >
                  {r.overallStatus}
                </span>
                {r.explainUsed && (
                  <span className="px-1 py-0.5 text-[10px] rounded bg-purple-100 text-purple-700 dark:bg-purple-900/40 dark:text-purple-300">
                    ex
                  </span>
                )}
              </div>
              <span
                className="font-mono text-[10px] opacity-60 truncate max-w-[140px]"
                title={r.signature}
              >
                {r.signature}
              </span>
              <span className="text-[10px] text-neutral-500 shrink-0">
                {formatTime(r.createdAt)}
              </span>
            </div>
            <div
              className="mt-1 text-[11px] text-neutral-600 dark:text-neutral-300 truncate"
              title={r.input}
            >
              {r.input}
            </div>
            {r.intents && r.intents.length > 0 && (
              <div className="mt-1 flex flex-wrap gap-1">
                {r.intents.slice(0, 6).map((it) => (
                  <span
                    key={it}
                    className="px-1 py-0.5 rounded bg-neutral-100 dark:bg-neutral-800 text-[10px] font-mono"
                    title={it}
                  >
                    {it}
                  </span>
                ))}
                {r.intents.length > 6 && (
                  <span className="text-[10px] opacity-60">+{r.intents.length - 6}</span>
                )}
              </div>
            )}
          </li>
        ))}
      </ul>
      {hasMore && (
        <button
          type="button"
          onClick={() => {
            if (!items.length) return;
            onNextPage?.(items[items.length - 1].createdAt);
          }}
          className="text-xs self-center px-3 py-1 rounded border border-neutral-300 dark:border-neutral-600 hover:bg-neutral-100 dark:hover:bg-neutral-800"
        >
          More…
        </button>
      )}
    </div>
  );
};

export default HistoryList;
