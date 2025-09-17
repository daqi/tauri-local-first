import { useEffect, useState } from 'react';
import commands from '@/commands';
import { Item } from '@/typing';
import { SYSTEM_HOST_ITEM } from '@/constants';

export default function useList() {
  const [list, setList] = useState<Item[]>([]);
  const [current, setCurrent] = useState<Item | null>(SYSTEM_HOST_ITEM);

  useEffect(() => {
    const init = async () => {
      const list: Item[] = await commands.getList();
      setList(list);
    };
    init();
  }, []);

  const updateList = async (next: Item[]) => {
    const res = await commands.setList(next);
    if (res) setList(next);
  };

  return {
    list: [SYSTEM_HOST_ITEM, ...list], // with system
    userList: list, // without system
    current,
    setCurrent,
    updateList: commands.setList, // expects userList array
    mutateList: async (fn: (items: Item[]) => Item[]) => {
      const next = fn(list);
      return updateList(next);
    },
    createItem: async (item: Item) => {
      const next = [...list, item];
      return updateList(next);
    },
    updateItem: async (id: string, data: Partial<Item>) => {
      // recursive update for tree
      const walk = (arr: Item[]): Item[] => arr.map(it => {
        if (it.id === id) return { ...it, ...data };
        if (it.children) return { ...it, children: walk(it.children) };
        return it;
      });
      const next = walk(list);
      return updateList(next);
    },
    deleteItem: async (id: string) => {
      const walk = (arr: Item[]): Item[] => arr.filter(it => {
        if (it.id === id) return false;
        if (it.children) it.children = walk(it.children);
        return true;
      });
      const next = walk(list);
      if (current?.id === id) {
        setCurrent(null); // let UI pick another later
      }
      return updateList(next);
    },
  };
}

export type UseListReturn = ReturnType<typeof useList>;
