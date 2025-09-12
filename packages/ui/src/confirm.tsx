import * as AlertDialog from '@radix-ui/react-alert-dialog';
import React from 'react';
import { createRoot } from 'react-dom/client';

type ConfirmOptions = {
  title?: string;
  description?: string;
  confirmText?: string;
  cancelText?: string;
};

const ConfirmModal: React.FC<{
  options: ConfirmOptions;
  onResolve: (ok: boolean) => void;
}> = ({ options, onResolve }) => {
  const { title = '确认操作', description, confirmText = '确定', cancelText = '取消' } = options;
  const [open, setOpen] = React.useState(true);
  const handleClose = (ok: boolean) => {
    onResolve(ok);
    setOpen(false);
  };
  return (
    <AlertDialog.Root open={open} onOpenChange={(o: boolean) => !o && onResolve(false)}>
      <AlertDialog.Portal>
        <AlertDialog.Overlay style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,.2)' }} />
        <AlertDialog.Content style={{
          position: 'fixed', top: '50%', left: '50%', transform: 'translate(-50%,-50%)',
          background: 'white', color: '#111', borderRadius: 8, padding: 16, width: 360,
          boxShadow: '0 10px 30px rgba(0,0,0,.15)'
        }}>
          <AlertDialog.Title style={{ fontSize: 16, fontWeight: 600 }}>{title}</AlertDialog.Title>
          {description ? (
            <AlertDialog.Description style={{ marginTop: 8, color: '#555', fontSize: 14 }}>
              {description}
            </AlertDialog.Description>
          ) : null}
          <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 8, marginTop: 16 }}>
            <AlertDialog.Cancel asChild>
              <button type="button" onClick={() => handleClose(false)}
                style={{ padding: '6px 12px', borderRadius: 6, border: '1px solid #ddd', background: '#f8f8f8' }}>
                {cancelText}
              </button>
            </AlertDialog.Cancel>
            <AlertDialog.Action asChild>
              <button type="button" onClick={() => handleClose(true)}
                style={{ padding: '6px 12px', borderRadius: 6, border: '1px solid #16a34a', background: '#22c55e', color: '#fff' }}>
                {confirmText}
              </button>
            </AlertDialog.Action>
          </div>
        </AlertDialog.Content>
      </AlertDialog.Portal>
    </AlertDialog.Root>
  );
};

export default function confirm(options: ConfirmOptions = {}): Promise<boolean> {
  return new Promise((resolve) => {
    const container = document.createElement('div');
    document.body.appendChild(container);
    const root = createRoot(container);
    const onResolve = (ok: boolean) => {
      resolve(ok);
      setTimeout(() => {
        try { root.unmount(); } catch {}
        container.parentElement?.removeChild(container);
      }, 0);
    };
    root.render(
      <ConfirmModal
        options={{ title: options.title ?? '确认操作', description: options.description, confirmText: options.confirmText, cancelText: options.cancelText }}
        onResolve={onResolve}
      />
    );
  });
}
