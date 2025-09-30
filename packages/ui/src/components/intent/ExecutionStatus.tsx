import React from 'react';
import { cn } from '../../lib/utils';

export interface ExecutionStatusProps {
  overallStatus?: string;
  cacheHit?: boolean;
  conflicts?: number;
  batches?: number;
  className?: string;
  explainUsed?: boolean;
  signature?: string | null;
}

const badgeColor = (s?: string) => {
  switch (s) {
    case 'success':
      return 'bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-300';
    case 'failed':
      return 'bg-red-100 text-red-700 dark:bg-red-900/40 dark:text-red-300';
    case 'partial':
      return 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/40 dark:text-yellow-300';
    default:
      return 'bg-neutral-100 text-neutral-700 dark:bg-neutral-800 dark:text-neutral-300';
  }
};

const ExecutionStatus: React.FC<ExecutionStatusProps> = ({
  overallStatus,
  cacheHit,
  conflicts,
  batches,
  className,
  explainUsed,
  signature,
}) => {
  return (
    <div className={cn('text-xs flex flex-wrap items-center gap-2', className)}>
      {overallStatus && (
        <span
          className={cn('px-2 py-0.5 rounded font-medium capitalize', badgeColor(overallStatus))}
        >
          {overallStatus}
        </span>
      )}
      {cacheHit && (
        <span className="px-1.5 py-0.5 rounded bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300">
          cache
        </span>
      )}
      {explainUsed && (
        <span className="px-1.5 py-0.5 rounded bg-purple-100 text-purple-700 dark:bg-purple-900/40 dark:text-purple-300">
          explain
        </span>
      )}
      {typeof conflicts === 'number' && conflicts > 0 && (
        <span className="px-1.5 py-0.5 rounded bg-orange-100 text-orange-700 dark:bg-orange-900/40 dark:text-orange-300">
          conflicts:{conflicts}
        </span>
      )}
      {typeof batches === 'number' && (
        <span className="px-1.5 py-0.5 rounded bg-neutral-200 dark:bg-neutral-700">
          batches:{batches}
        </span>
      )}
      {signature && (
        <span className="truncate max-w-[160px] font-mono text-[10px] opacity-60" title={signature}>
          {signature}
        </span>
      )}
    </div>
  );
};

export default ExecutionStatus;
