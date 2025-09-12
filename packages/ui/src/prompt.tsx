import React, { useEffect, useRef } from 'react';
import { createRoot } from 'react-dom/client';
import * as Dialog from '@radix-ui/react-dialog';

export type PromptOptions = {
  title?: string;
  description?: string;
  placeholder?: string;
  confirmText?: string;
  cancelText?: string;
  inputType?: React.HTMLInputTypeAttribute; // 'password' | 'text' | 'email' | ...
  defaultValue?: string;
};

export default function promptPassword(options: PromptOptions = {}): Promise<string | undefined> {
  const {
    title = '请输入',
    description = '',
    placeholder = '输入内容',
    confirmText = '确认',
    cancelText = '取消',
    inputType = 'password',
    defaultValue = ''
  } = options;

  return new Promise((resolve) => {
    const container = document.createElement('div');
    document.body.appendChild(container);
    const root = createRoot(container);

    const Modal: React.FC = () => {
      const inputRef = useRef<HTMLInputElement | null>(null);
      useEffect(() => { inputRef.current?.focus(); }, []);
      const [open] = React.useState(true);
      const doResolve = (v?: string) => {
        resolve(v);
        setTimeout(() => { try { root.unmount(); } catch {}; container.parentElement?.removeChild(container); }, 0);
      };
      return (
        <Dialog.Root open={open} onOpenChange={(o) => !o && doResolve(undefined)}>
          <Dialog.Portal>
            <Dialog.Overlay style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,.2)' }} />
            <Dialog.Content style={{
              position: 'fixed', top: '50%', left: '50%', transform: 'translate(-50%,-50%)',
              background: 'white', color: '#111', borderRadius: 8, padding: 16, width: 420,
              boxShadow: '0 10px 30px rgba(0,0,0,.15)'
            }}>
              <Dialog.Title style={{ fontSize: 16, fontWeight: 600 }}>{title}</Dialog.Title>
              {description ? (
                <Dialog.Description style={{ marginTop: 8, color: '#555', fontSize: 14 }}>
                  {description}
                </Dialog.Description>
              ) : null}
              <input
                ref={inputRef}
                type={inputType}
                placeholder={placeholder}
                defaultValue={defaultValue}
                onKeyDown={(e) => { if (e.key === 'Enter') { e.preventDefault(); doResolve(inputRef.current?.value || undefined); } }}
                style={{ marginTop: 12, width: '100%', padding: '8px 10px', border: '1px solid #ddd', borderRadius: 6 }}
              />
              <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 8, marginTop: 16 }}>
                <Dialog.Close asChild>
                  <button type="button" onClick={() => doResolve(undefined)}
                    style={{ padding: '6px 12px', borderRadius: 6, border: '1px solid #ddd', background: '#f8f8f8' }}>
                    {cancelText}
                  </button>
                </Dialog.Close>
                <button type="button" onClick={() => doResolve(inputRef.current?.value || undefined)}
                  style={{ padding: '6px 12px', borderRadius: 6, border: '1px solid #2563eb', background: '#3b82f6', color: '#fff' }}>
                  {confirmText}
                </button>
              </div>
            </Dialog.Content>
          </Dialog.Portal>
        </Dialog.Root>
      );
    };

    root.render(<Modal />);
  });
}
