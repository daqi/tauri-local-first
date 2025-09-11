import React, { useEffect, useRef } from 'react';
import { createRoot, Root } from 'react-dom/client';
import styles from './index.module.less';

type ModalProps = { onResolve: (v?: string) => void };

const Modal: React.FC<ModalProps> = ({ onResolve }) => {
  const inputRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        onResolve(inputRef.current?.value || undefined);
      } else if (e.key === 'Escape') {
        e.preventDefault();
        onResolve(undefined);
      }
    };
    document.addEventListener('keydown', onKey);
    // focus input after mount
    inputRef.current?.focus();
    return () => document.removeEventListener('keydown', onKey);
  }, []);

  return (
    <div className={styles.overlay} role="dialog" aria-modal="true">
      <div className={styles.modal}>
        <div className={styles.title}>请输入系统密码</div>
        <div className={styles.desc}>为写入系统 hosts，请输入管理员密码：</div>
        <input
          ref={inputRef}
          type="password"
          placeholder="系统密码"
          className={styles.input}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              e.preventDefault();
              onResolve(inputRef.current?.value || undefined);
            }
          }}
        />
        <div className={styles.actions}>
          <button
            type="button"
            className={styles.cancelBtn}
            onClick={() => onResolve(undefined)}
          >
            取消
          </button>
          <button
            type="button"
            className={styles.okBtn}
            onClick={() => onResolve(inputRef.current?.value || undefined)}
          >
            确定
          </button>
        </div>
      </div>
    </div>
  );
};

const promptSystemPassword = (): Promise<string | undefined> => {
  return new Promise((resolve) => {
    const container = document.createElement('div');
    document.body.appendChild(container);
    const root: Root = createRoot(container);

    let resolved = false;
    const doResolve = (value?: string) => {
      if (resolved) return;
      resolved = true;
      resolve(value);
      // give React a tick to finish handlers, then cleanup
      setTimeout(() => {
        try {
          root.unmount();
        } catch (e) {
          /* ignore */
        }
        if (container.parentElement) {
          container.parentElement.removeChild(container);
        }
      }, 0);
    };

    root.render(<Modal onResolve={doResolve} />);
  });
};

export default promptSystemPassword;
