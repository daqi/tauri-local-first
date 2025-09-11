import { ReactNode } from 'react';
import styles from './index.module.less';

export default function Main({ children }: { children: ReactNode }) {
  return <div className={styles.main}>{children}</div>;
}
