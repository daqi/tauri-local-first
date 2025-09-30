import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import IntentPanel from '../modules/intent/IntentPanel';
import { useIntentStore } from '../modules/intent/store';

describe('IntentPanel integration', () => {
  beforeEach(() => {
    // reset store
    const { setState } = useIntentStore as any;
    setState({ input: '', plan: undefined, lastRun: undefined, history: [] });
  });

  it('parse -> dry run -> execute updates UI', async () => {
    render(<IntentPanel />);
    const textarea = screen.getByPlaceholderText(/enter intent/i);
    fireEvent.change(textarea, { target: { value: 'hosts:switch(dev)' } });
    fireEvent.click(screen.getByText(/parse/i));
    await waitFor(() => expect(screen.getByText(/batches:1/i)).toBeInTheDocument());

    fireEvent.click(screen.getByText(/dry run/i));
    await waitFor(() => expect(screen.getAllByText(/simulated/i).length).toBeGreaterThan(0));

    fireEvent.click(screen.getByText(/execute/i));
    await waitFor(() => expect(screen.getAllByText(/success/i).length).toBeGreaterThan(0));

    // history section
    await waitFor(() => expect(screen.getByText(/history/i)).toBeInTheDocument());
  });
});
