import React from 'react';
import { cn } from '../../lib/utils';

export interface IntentActionItem {
  intentId: string;
  status: string;
  reason?: string;
  predictedEffects?: any;
  durationMs?: number;
}

export interface ActionListProps {
  actions: IntentActionItem[];
  className?: string;
  compact?: boolean;
}

export const statusColor = (s: string) => {
  switch (s) {
    case 'success':
      return 'text-green-600 dark:text-green-400';
    case 'failed':
      return 'text-red-600 dark:text-red-400';
    case 'timeout':
      return 'text-orange-600 dark:text-orange-400';
    case 'simulated':
      return 'text-blue-600 dark:text-blue-400';
    case 'partial':
      return 'text-yellow-600 dark:text-yellow-400';
    default:
      return 'text-neutral-600 dark:text-neutral-400';
  }
};

const ActionList: React.FC<ActionListProps> = ({ actions, className, compact }) => {
  if (!actions.length) {
    return <div className={cn('text-sm text-neutral-500 italic', className)}>No actions</div>;
  }
  return (
    <ul className={cn('space-y-1 text-sm', className)}>
      {actions.map(a => (
        <li
          key={a.intentId}
          className={cn(
            'border rounded px-2 py-1 flex flex-col gap-0.5 bg-white/60 dark:bg-neutral-900/60 backdrop-blur',
            'border-neutral-200 dark:border-neutral-700'
          )}
        >
          <div className="flex items-center justify-between gap-2">
            <span className="font-mono text-xs truncate" title={a.intentId}>{a.intentId}</span>
            <span className={cn('font-medium', statusColor(a.status))}>{a.status}</span>
          </div>
          {!compact && a.reason && (
            <div className="text-xs text-red-500 dark:text-red-400 break-words" title={a.reason}>
              {a.reason}
            </div>
          )}
          {!compact && a.durationMs != null && (
            <div className="text-[10px] text-neutral-500">{a.durationMs} ms</div>
          )}
        </li>
      ))}
    </ul>
  );
};

export default ActionList;
