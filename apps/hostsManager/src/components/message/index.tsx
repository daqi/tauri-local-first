import React, { useEffect, useRef } from 'react';
import { createRoot, Root } from 'react-dom/client';
import styles from './index.module.less';
type MessageType = 'success' | 'error' | 'warning';

interface NoticeProps {
  type: MessageType;
  content: React.ReactNode;
  duration?: number;
  onClose?: () => void;
}

const Notice: React.FC<NoticeProps> = ({ type, content, duration = 3000, onClose }) => {
  const timerRef = useRef<number | null>(null);

  useEffect(() => {
    if (duration > 0) {
      timerRef.current = window.setTimeout(() => {
        onClose?.();
      }, duration);
    }
    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
    };
  }, [duration, onClose]);

  const icon = (() => {
    switch (type) {
      case 'success':
        return '✓';
      case 'error':
        return '✕';
      case 'warning':
        return '!';
      default:
        return '';
    }
  })();

  return (
    <div className={`${styles.notice} ${styles[type] || ''}`}>
      <span className={styles.icon}>{icon}</span>
      <div className={styles.content}>{content}</div>
    </div>
  );
};

function createMessage(type: MessageType, content: React.ReactNode, duration = 3000) {
  const container = document.createElement('div');
  document.body.appendChild(container);
  const root: Root = createRoot(container);

  const close = () => {
    try {
      root.unmount();
    } finally {
      if (container.parentNode) container.parentNode.removeChild(container);
    }
  };

  root.render(<Notice type={type} content={content} duration={duration} onClose={close} />);

  return { close };
}

const message = {
  success(content: React.ReactNode, duration?: number) {
    return createMessage('success', content, duration);
  },
  error(content: React.ReactNode, duration?: number) {
    return createMessage('error', content, duration);
  },
  warning(content: React.ReactNode, duration?: number) {
    return createMessage('warning', content, duration);
  },
};

export default message;
