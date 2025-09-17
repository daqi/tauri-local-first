import { Item } from '../typing';
import { v4 as uuidV4 } from 'uuid';

// Helper: deep clone simple
const clone = <T>(v: T): T => JSON.parse(JSON.stringify(v));

export type TreePath = string[]; // sequence of ids

export function findItem(items: Item[], id: string): Item | null {
  for (const it of items) {
    if (it.id === id) return it;
    if (it.children) {
      const f = findItem(it.children, id);
      if (f) return f;
    }
  }
  return null;
}

export function addChild(items: Item[], parentId: string | null, node: Item): Item[] {
  const next = clone(items);
  if (!parentId) {
    next.push(node);
    return next;
  }
  const walk = (arr: Item[]): boolean => {
    for (const it of arr) {
      if (it.id === parentId) {
        if (it.type !== 'folder') {
          it.type = 'folder';
          it.children = it.children || [];
        }
        it.children = it.children || [];
        it.children.push(node);
        return true;
      }
      if (it.children && walk(it.children)) return true;
    }
    return false;
  };
  walk(next);
  return next;
}

export function removeItem(items: Item[], id: string): Item[] {
  const next = clone(items);
  const walk = (arr: Item[]): Item[] => {
    return arr.filter(it => {
      if (it.id === id) return false;
      if (it.children) it.children = walk(it.children);
      return true;
    });
  };
  return walk(next);
}

export function updateItemName(items: Item[], id: string, name: string): Item[] {
  const next = clone(items);
  const walk = (arr: Item[]) => {
    for (const it of arr) {
      if (it.id === id) {
        it.name = name;
      }
      if (it.children) walk(it.children);
    }
  };
  walk(next);
  return next;
}

export function toggleItemOn(items: Item[], id: string, on: boolean): Item[] {
  const next = clone(items);
  const walk = (arr: Item[]) => {
    for (const it of arr) {
      if (it.id === id) {
        it.on = on;
      }
      if (it.children) walk(it.children);
    }
  };
  walk(next);
  return next;
}

export function createFile(name = '新建规则'): Item {
  return { id: uuidV4(), name, on: true, type: 'file' };
}
export function createFolder(name = '新建文件夹'): Item {
  return { id: uuidV4(), name, on: true, type: 'folder', children: [] };
}

export function duplicateSubtree(node: Item): Item {
  const copy = clone(node);
  const remap = (n: Item) => {
    n.id = uuidV4();
    if (n.children) n.children.forEach(remap);
  };
  remap(copy);
  return copy;
}

// Paste (copy) or move (cut) under a folder (or root parentId null)
export function pasteNode(items: Item[], targetFolderId: string | null, node: Item): Item[] {
  const next = clone(items);
  const newNode = duplicateSubtree(node);
  return addChild(next, targetFolderId, newNode);
}

// Move (cut) by removing original then addChild (keeping id?) -> we keep ids stable when moving
export function moveNode(items: Item[], nodeId: string, targetFolderId: string | null): Item[] {
  const next = clone(items);
  let moving: Item | null = null;
  const extract = (arr: Item[]): Item[] => {
    return arr.filter(it => {
      if (it.id === nodeId) {
        moving = it;
        return false;
      }
      if (it.children) it.children = extract(it.children);
      return true;
    });
  };
  const stripped = extract(next);
  if (!moving) return items; // not found
  return addChild(stripped, targetFolderId, moving);
}
