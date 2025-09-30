import { describe, it, expect, vi } from 'vitest';
import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import HistoryList from '../HistoryList';

describe('HistoryList', () => {
  it('renders empty', () => {
    render(<HistoryList items={[]} />);
    expect(screen.getByText(/No history/)).toBeTruthy();
  });
  it('renders items and loads more', () => {
    const items = [
      {
        signature: 'sig1',
        input: 'cmd one',
        overallStatus: 'success',
        createdAt: Date.now(),
        planSize: 2,
      },
    ];
    const handler = vi.fn();
    render(<HistoryList items={items} hasMore onNextPage={handler} />);
    fireEvent.click(screen.getByText(/More…/));
    expect(handler).toHaveBeenCalled();
  });
});
