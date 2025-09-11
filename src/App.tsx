import './App.css';
import { StoreProvider } from './store';
import Main from './components/Main';
import List from './components/List';
import Editor from './components/Editor';

function App() {
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
