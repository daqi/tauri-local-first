import { describe, it, expect } from 'vitest';
import React from 'react';
import { render, screen } from '@testing-library/react';
import ActionList, { IntentActionItem } from '../ActionList';

describe('ActionList', () => {
  it('renders empty state', () => {
    render(<ActionList actions={[]} />);
    expect(screen.getByText(/No actions/i)).toBeTruthy();
  });
  it('renders actions with status colors', () => {
    const actions: IntentActionItem[] = [
      { intentId: 'a1', status: 'success' },
      { intentId: 'a2', status: 'failed', reason: 'boom' },
    ];
    render(<ActionList actions={actions} />);
    expect(screen.getByText('a1')).toBeTruthy();
    expect(screen.getByText('a2')).toBeTruthy();
    expect(screen.getByText('failed')).toBeTruthy();
  });
});
