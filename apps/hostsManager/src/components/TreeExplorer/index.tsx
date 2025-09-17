import { useEffect, useRef, useState } from 'react';
import { useStore } from '../../store';
import styles from './index.module.less';
import { Item } from '../../typing';
import * as Checkbox from '@radix-ui/react-checkbox';
import { CheckIcon, ChevronDownIcon, ChevronRightIcon, FileIcon } from '@radix-ui/react-icons';
import { createFile, createFolder, addChild, removeItem, updateItemName, toggleItemOn, pasteNode, moveNode } from '../../utils/treeOps';
import writeHostsToSystem from '../../utils/writeHostsToSystem';
import { confirm } from '@suite/ui';

interface ClipboardState { mode: 'copy' | 'cut'; node: Item };

function TreeNode({ node, depth, activeId, onSelect, onToggle, onRename, onCheck, expanded, setExpanded, onContextMenu } : { node: Item; depth: number; activeId: string | null; onSelect: (n: Item) => void; onToggle: (id: string) => void; onRename: (id: string, name: string) => void; onCheck: (id: string, on: boolean) => void; expanded: Set<string>; setExpanded: React.Dispatch<React.SetStateAction<Set<string>>>; onContextMenu: (e: React.MouseEvent, node: Item) => void; }) {
  const isFolder = node.type === 'folder';
  const isOpen = isFolder && expanded.has(node.id);
  const [renaming, setRenaming] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  useEffect(() => { if (renaming && inputRef.current) { inputRef.current.focus(); inputRef.current.select(); } }, [renaming]);
  return (
    <div>
      <div
        className={`${styles.nodeRow} ${activeId === node.id ? styles.nodeActive : ''}`}
        style={{ paddingLeft: depth * 12 }}
        onClick={() => onSelect(node)}
        onContextMenu={(e) => onContextMenu(e, node)}
      >
        {isFolder ? (
          <span className={styles.toggle} onClick={(e) => { e.stopPropagation(); onToggle(node.id); }}>{isOpen ? <ChevronDownIcon /> : <ChevronRightIcon />}</span>
        ) : <span className={styles.toggle}></span>}
        <Checkbox.Root
          className={styles.checkbox}
          checked={node.on}
          onCheckedChange={(checked: any) => onCheck(node.id, checked === true)}
          onClick={(e) => e.stopPropagation()}
        >
          <Checkbox.Indicator>
            <CheckIcon width={12} height={12} />
          </Checkbox.Indicator>
        </Checkbox.Root>
  {isFolder ? (isOpen ? <span style={{width:14}}>📂</span> : <span style={{width:14}}>📁</span>) : <FileIcon />}
        {renaming ? (
          <input
            ref={inputRef}
            defaultValue={node.name}
            className={styles.inputRename}
            onClick={(e) => e.stopPropagation()}
            onKeyDown={(e) => {
              if (e.key === 'Enter') { onRename(node.id, (e.target as HTMLInputElement).value.trim() || node.name); setRenaming(false); }
              if (e.key === 'Escape') setRenaming(false);
            }}
            onBlur={(e) => { onRename(node.id, (e.target as HTMLInputElement).value.trim() || node.name); setRenaming(false); }}
          />
        ) : (
          <span style={{ marginLeft: 4 }} onDoubleClick={() => { if (!isFolder) setRenaming(true); else onToggle(node.id); }}>{node.name}</span>
        )}
        <span style={{ display: 'flex', gap: 4, marginLeft: 'auto' }}>
          <button style={{ visibility: 'hidden' }} onClick={(e) => e.stopPropagation()} aria-hidden />
        </span>
      </div>
      {isFolder && isOpen && node.children?.map(child => (
        <TreeNode key={child.id} node={child} depth={depth + 1} activeId={activeId} onSelect={onSelect} onToggle={onToggle} onRename={onRename} onCheck={onCheck} expanded={expanded} setExpanded={setExpanded} onContextMenu={onContextMenu} />
      ))}
    </div>
  );
}

export default function TreeExplorer() {
  // store: userList (tree), current selected file item, mutateList for immutable updates
  const { userList, current, setCurrent, mutateList } = useStore() as any; // cast for extended fields
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [clipboard, setClipboard] = useState<ClipboardState | null>(null);
  const [menu, setMenu] = useState<{ x: number; y: number; target: Item | null; forRoot?: boolean } | null>(null);

  const closeMenu = () => setMenu(null);
  useEffect(() => { const h = () => closeMenu(); window.addEventListener('click', h); return () => window.removeEventListener('click', h); }, []);

  const apply = async (next: Item[]) => { await mutateList(() => next); };

  const handleContext = (e: React.MouseEvent, node: Item) => {
    e.preventDefault();
    setMenu({ x: e.clientX, y: e.clientY, target: node });
  };

  const handleRootContext = (e: React.MouseEvent) => {
    e.preventDefault();
    setMenu({ x: e.clientX, y: e.clientY, target: null, forRoot: true });
  };

  const createIn = async (parent: Item | null, kind: 'file' | 'folder') => {
    const n = kind === 'file' ? createFile() : createFolder();
    const next = addChild(userList, parent ? parent.id : null, n);
    await apply(next);
    if (kind === 'file') setCurrent(n);
    if (parent && parent.type === 'folder') setExpanded(s => new Set(s).add(parent.id));
  };

  const rename = async (id: string, name: string) => {
    if (!name) return;
    await apply(updateItemName(userList, id, name));
  };

  const remove = async (node: Item) => {
    if (node.system) return;
    if (await confirm({ title: '删除', description: '确定删除该节点及其所有子节点？'})) {
      await apply(removeItem(userList, node.id));
      if (current?.id === node.id) setCurrent(null);
      await writeHostsToSystem();
    }
  };

  const toggle = async (id: string) => {
    const isOpen = expanded.has(id);
    setExpanded(s => { const ns = new Set(s); if (isOpen) ns.delete(id); else ns.add(id); return ns; });
  };

  const setOn = async (id: string, on: boolean) => {
    await apply(toggleItemOn(userList, id, on));
    await writeHostsToSystem();
  };

  const doCopy = (node: Item, mode: 'copy' | 'cut') => {
    setClipboard({ mode, node });
  };

  const doPaste = async (targetFolder: Item | null) => {
    if (!clipboard) return;
    if (clipboard.mode === 'copy') {
      const next = pasteNode(userList, targetFolder ? targetFolder.id : null, clipboard.node);
      await apply(next);
    } else {
      // cut -> move
      const next = moveNode(userList, clipboard.node.id, targetFolder ? targetFolder.id : null);
      await apply(next);
      setClipboard(null);
    }
  };

  const menuActions = () => {
    if (!menu) return null;
    const { target, forRoot } = menu;
    const isFolder = target?.type === 'folder';
    const canPasteHere = !!clipboard; // root or folder
    return (
      <div className={styles.contextMenu} style={{ left: menu.x, top: menu.y }}>
        {(forRoot || isFolder) && <button onClick={() => { createIn(target || null, 'file'); closeMenu(); }}>新建文件</button>}
        {(forRoot || isFolder) && <button onClick={() => { createIn(target || null, 'folder'); closeMenu(); }}>新建文件夹</button>}
        {target && <button onClick={() => { doCopy(target, 'copy'); closeMenu(); }}>复制</button>}
        {target && <button onClick={() => { doCopy(target, 'cut'); closeMenu(); }}>剪切</button>}
        {(forRoot || isFolder) && canPasteHere && <button onClick={() => { doPaste(target || null); closeMenu(); }}>粘贴</button>}
  {target && !target.system && <button onClick={() => { /* 用户可双击名称重命名，此处可后续实现触发 */ closeMenu(); }}>重命名(双击名称)</button>}
        {target && !target.system && <button onClick={() => { remove(target); closeMenu(); }}>删除</button>}
      </div>
    );
  };

  return (
    <div className={styles.wrapper} onContextMenu={handleRootContext}>
      <div className={styles.treeRoot}>
        {userList.length === 0 && <div className={styles.emptyHint}>右键空白新建文件或文件夹</div>}
        {userList.map((it: Item) => (
          <TreeNode key={it.id} node={it} depth={0} activeId={current?.id || null} onSelect={(n) => setCurrent(n.type === 'file' ? n : current)} onToggle={toggle} onRename={rename} onCheck={setOn} expanded={expanded} setExpanded={setExpanded} onContextMenu={handleContext} />
        ))}
      </div>
      {menuActions()}
    </div>
  );
}
