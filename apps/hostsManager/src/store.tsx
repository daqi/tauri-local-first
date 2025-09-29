import { createContext, useContext } from 'react';
import { ReactNode } from 'react';
import useList, { UseListReturn } from './hooks/useList';
import useContent, { UseContentReturn } from './hooks/useContent';
type StoreContextType = Omit<UseListReturn, 'fetchList'> &
    UseContentReturn & {
        // backward note: list includes system; userList excludes system
    };

const StoreContext = createContext({} as StoreContextType);

export function useStore() {
    return useContext(StoreContext);
}

export function StoreProvider({ children }: { children?: ReactNode }) {
    const {
        list,
        userList,
        current,
        setCurrent,
        updateList,
        mutateList,
        createItem,
        updateItem,
        deleteItem,
    } = useList();

    const { content, contentId, updateContent } = useContent(current?.id);

    return (
        <StoreContext.Provider
            value={{
                list,
                userList,
                mutateList,
                current,
                setCurrent,
                updateList,
                createItem,
                updateItem,
                deleteItem,
                content,
                contentId,
                updateContent,
            }}
        >
            {children}
        </StoreContext.Provider>
    );
}
