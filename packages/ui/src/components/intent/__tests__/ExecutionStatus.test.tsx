import { describe, it, expect } from 'vitest';
import React from 'react';
import { render, screen } from '@testing-library/react';
import ExecutionStatus from '../ExecutionStatus';

describe('ExecutionStatus', () => {
  it('shows overall status badge', () => {
    render(<ExecutionStatus overallStatus="success" batches={1} conflicts={0} cacheHit />);
    expect(screen.getByText(/success/i)).toBeTruthy();
    expect(screen.getByText(/batches:1/i)).toBeTruthy();
  });
});
