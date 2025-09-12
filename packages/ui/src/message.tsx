import React from 'react';
import { createRoot } from 'react-dom/client';
import * as Toast from '@radix-ui/react-toast';

type MessageType = 'success' | 'error' | 'warning';

const colors: Record<MessageType, { bg: string; fg: string; border: string; icon: string }> = {
  success: { bg: '#ecfdf5', fg: '#065f46', border: '#bbf7d0', icon: '✓' },
  error: { bg: '#fef2f2', fg: '#991b1b', border: '#fecaca', icon: '✕' },
  warning: { bg: '#fffbeb', fg: '#92400e', border: '#fde68a', icon: '!' },
};

const Host: React.FC<{ type: MessageType; content: React.ReactNode; duration?: number; onClose: () => void }>=({ type, content, duration = 3000, onClose })=>{
  const [open, setOpen] = React.useState(true);
  React.useEffect(() => {
    const t = setTimeout(() => setOpen(false), duration);
    return () => clearTimeout(t);
  }, [duration]);
  React.useEffect(() => { if (!open) onClose(); }, [open]);
  const c = colors[type];
  return (
    <Toast.Provider swipeDirection="right">
      <Toast.Root open={open} onOpenChange={setOpen}
        style={{
          background: c.bg, color: c.fg, border: `1px solid ${c.border}`,
          borderRadius: 8, padding: '10px 12px', display: 'flex', gap: 8,
          alignItems: 'center', boxShadow: '0 10px 20px rgba(0,0,0,.08)'
        }}>
        <div style={{ fontWeight: 700 }}>{c.icon}</div>
        <div style={{ fontSize: 14 }}>{content}</div>
      </Toast.Root>
      <Toast.Viewport
        style={{ position: 'fixed', right: 12, bottom: 12, display: 'flex', flexDirection: 'column', gap: 8, width: 360, maxWidth: '100vw', outline: 'none' }}
      />
    </Toast.Provider>
  );
};

function createMessage(type: MessageType, content: React.ReactNode, duration = 3000) {
  const el = document.createElement('div');
  document.body.appendChild(el);
  const root = createRoot(el);
  const cleanup = () => {
    try { root.unmount(); } catch {}
    el.parentElement?.removeChild(el);
  };
  root.render(<Host type={type} content={content} duration={duration} onClose={cleanup} />);
  return { close: cleanup };
}

const message = {
  success(content: React.ReactNode, duration?: number) { return createMessage('success', content, duration); },
  error(content: React.ReactNode, duration?: number) { return createMessage('error', content, duration); },
  warning(content: React.ReactNode, duration?: number) { return createMessage('warning', content, duration); },
};

export default message;
