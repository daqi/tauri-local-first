import './App.css';
import { StoreProvider } from './store';
import Main from './components/Main';
import List from './components/List';
import Editor from './components/Editor';
import { useEffect } from 'react';
import { getCurrent, onOpenUrl } from '@tauri-apps/plugin-deep-link';
import { getCurrentWindow } from '@tauri-apps/api/window';

function App() {
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      const urls = await getCurrent();
      if (urls && urls.length) {
        try { await getCurrentWindow().setFocus(); } catch {}
      }
      unlisten = await onOpenUrl(async (urls) => {
        try { await getCurrentWindow().setFocus(); } catch {}
        // simple parse: tlfsuite://open?app=hostsManager[&args=...]
        try {
          const u = new URL(urls[0]);
          const app = u.searchParams.get('app');
          const args = u.searchParams.get('args');
          console.log('deep link ->', { app, args });
          // TODO: 根据 app/args 执行内部导航或行为
        } catch {}
      });
    })();
    return () => { if (unlisten) unlisten(); };
  }, []);
  return (
    <StoreProvider>
      <Main>
        <List />
        <Editor />
      </Main>
    </StoreProvider>
  );
}

export default App;
