import React from 'react';
import ReactDOM from 'react-dom/client';
import { Theme, ThemePanel } from '@radix-ui/themes';
import '@radix-ui/themes/styles.css';
import App from './modules/App';

ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
        <Theme appearance="dark" accentColor="indigo" grayColor="slate" radius="medium">
            <App />
            {/* 开发阶段可打开 ThemePanel 观察主题变量 */}
            {process.env.NODE_ENV === 'development' && <ThemePanel />}
        </Theme>
    </React.StrictMode>,
);
