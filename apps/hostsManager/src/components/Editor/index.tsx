import { useStore } from '@/store';
import CodeMirror, { ReactCodeMirrorRef } from '@uiw/react-codemirror';
import { useSize } from 'ahooks';
import styles from './index.module.less';
import { useRef } from 'react';
import hosts from './extensions/lang-hosts';

export default function Editor() {
  const { current, content, contentId, updateContent } = useStore();
  const ref = useRef<HTMLDivElement>(null);
  const editorRef = useRef<ReactCodeMirrorRef>(null);
  const size = useSize(ref);

  return (
    <div className={styles.editor} ref={ref}>
      <CodeMirror
        key={contentId}
        onChange={(value) => {
          if (contentId) {
            updateContent(contentId, value);
          }
        }}
        readOnly={current?.system}
        value={content || ''}
        height={(size?.height ?? 100) + 'px'}
        extensions={[hosts()]}
        ref={editorRef}
      />
    </div>
  );
}
