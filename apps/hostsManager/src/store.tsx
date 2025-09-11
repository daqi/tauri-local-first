import { createContext, useContext } from 'react';
import { ReactNode } from 'react';
import useList, { UseListReturn } from './hooks/useList';
import useContent, { UseContentReturn } from './hooks/useContent';

type StoreContextType = Omit<UseListReturn, 'fetchList'> & UseContentReturn;

const StoreContext = createContext({} as StoreContextType);

export function useStore() {
  return useContext(StoreContext);
}

export function StoreProvider({ children }: { children?: ReactNode }) {
  const { list, current, setCurrent, updateList, createItem, updateItem, deleteItem } =
    useList();

  const { content, contentId, updateContent } = useContent(
    current?.id
  );

  return (
    <StoreContext.Provider
      value={{
        list,
        current,
        setCurrent,
        updateList,
        createItem,
        updateItem,
        deleteItem,
        content,
        contentId,
        updateContent
      }}
    >
      {children}
    </StoreContext.Provider>
  );
}
