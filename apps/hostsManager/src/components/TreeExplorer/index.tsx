import { useEffect, useRef, useState } from 'react';
import { useStore } from '../../store';
import styles from './index.module.less';
import { Item } from '../../typing';
import * as Checkbox from '@radix-ui/react-checkbox';
import { CheckIcon, ChevronDownIcon, ChevronRightIcon, FileIcon } from '@radix-ui/react-icons';
import {
    createFile,
    createFolder,
    addChild,
    removeItem,
    updateItemName,
    toggleItemOn,
    pasteNode,
    moveNode,
} from '../../utils/treeOps';
import writeHostsToSystem from '../../utils/writeHostsToSystem';
import { confirm } from '@suite/ui';
import { Menu, MenuItem } from '@tauri-apps/api/menu';

interface ClipboardState {
    mode: 'copy' | 'cut';
    node: Item;
}

function TreeNode({
    node,
    depth,
    activeId,
    onSelect,
    onToggle,
    onRename,
    onCheck,
    expanded,
    setExpanded,
    onContextMenu,
    renamingId,
    setRenamingId,
}: {
    node: Item;
    depth: number;
    activeId: string | null;
    onSelect: (n: Item) => void;
    onToggle: (id: string) => void;
    onRename: (id: string, name: string) => void;
    onCheck: (id: string, on: boolean) => void;
    expanded: Set<string>;
    setExpanded: React.Dispatch<React.SetStateAction<Set<string>>>;
    onContextMenu: (e: React.MouseEvent, node: Item) => void;
    renamingId: string | null;
    setRenamingId: React.Dispatch<React.SetStateAction<string | null>>;
}) {
    const isFolder = node.type === 'folder';
    const isOpen = isFolder && expanded.has(node.id);
    const renaming = renamingId === node.id;
    const inputRef = useRef<HTMLInputElement>(null);
    useEffect(() => {
        if (renaming && inputRef.current) {
            inputRef.current.focus();
            inputRef.current.select();
        }
    }, [renaming]);
    return (
        <div>
            <div
                className={`${styles.nodeRow} ${activeId === node.id ? styles.nodeActive : ''}`}
                style={{ paddingLeft: depth * 12 }}
                onClick={() => onSelect(node)}
                onContextMenu={(e) => onContextMenu(e, node)}
            >
                {isFolder ? (
                    <span
                        className={styles.toggle}
                        onClick={(e) => {
                            e.stopPropagation();
                            onToggle(node.id);
                        }}
                    >
                        {isOpen ? <ChevronDownIcon /> : <ChevronRightIcon />}
                    </span>
                ) : (
                    <span className={styles.toggle}></span>
                )}
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
                {isFolder ? (
                    isOpen ? (
                        <span style={{ width: 14 }}>📂</span>
                    ) : (
                        <span style={{ width: 14 }}>📁</span>
                    )
                ) : (
                    <FileIcon />
                )}
                {renaming ? (
                    <input
                        ref={inputRef}
                        defaultValue={node.name}
                        className={styles.inputRename}
                        onClick={(e) => e.stopPropagation()}
                        onKeyDown={(e) => {
                            if (e.key === 'Enter') {
                                onRename(
                                    node.id,
                                    (e.target as HTMLInputElement).value.trim() || node.name,
                                );
                                setRenamingId(null);
                            }
                            if (e.key === 'Escape') setRenamingId(null);
                        }}
                        onBlur={(e) => {
                            onRename(
                                node.id,
                                (e.target as HTMLInputElement).value.trim() || node.name,
                            );
                            setRenamingId(null);
                        }}
                    />
                ) : (
                    <span
                        style={{ marginLeft: 4 }}
                        onDoubleClick={() => {
                            if (isFolder) onToggle(node.id);
                        }}
                    >
                        {node.name}
                    </span>
                )}
                <span style={{ display: 'flex', gap: 4, marginLeft: 'auto' }}>
                    <button
                        style={{ visibility: 'hidden' }}
                        onClick={(e) => e.stopPropagation()}
                        aria-hidden
                    />
                </span>
            </div>
            {isFolder &&
                isOpen &&
                node.children?.map((child) => (
                    <TreeNode
                        key={child.id}
                        node={child}
                        depth={depth + 1}
                        activeId={activeId}
                        onSelect={onSelect}
                        onToggle={onToggle}
                        onRename={onRename}
                        onCheck={onCheck}
                        expanded={expanded}
                        setExpanded={setExpanded}
                        onContextMenu={onContextMenu}
                        renamingId={renamingId}
                        setRenamingId={setRenamingId}
                    />
                ))}
        </div>
    );
}

export default function TreeExplorer() {
    // store: userList (tree), current selected file item, mutateList for immutable updates
    const { userList, current, setCurrent, mutateList } = useStore() as any; // cast for extended fields
    const [expanded, setExpanded] = useState<Set<string>>(new Set());
    const [clipboard, setClipboard] = useState<ClipboardState | null>(null);
    const [renamingId, setRenamingId] = useState<string | null>(null);

    const apply = async (next: Item[]) => {
        await mutateList(() => next);
    };

    const openNativeMenu = async (e: React.MouseEvent, target: Item | null) => {
        e.preventDefault();
        const forRoot = !target;
        const isFolder = target?.type === 'folder';
        const canPasteHere = !!clipboard && (forRoot || isFolder);

        const items: MenuItem[] = [];

        if (forRoot || isFolder) {
            items.push(
                await MenuItem.new({
                    id: 'new_file',
                    text: '新建文件',
                    action: () => {
                        createIn(target || null, 'file');
                    },
                }),
            );
            items.push(
                await MenuItem.new({
                    id: 'new_folder',
                    text: '新建文件夹',
                    action: () => {
                        createIn(target || null, 'folder');
                    },
                }),
            );
        }

        if (target) {
            items.push(
                await MenuItem.new({
                    id: 'copy',
                    text: '复制',
                    action: () => {
                        doCopy(target, 'copy');
                    },
                }),
            );
            items.push(
                await MenuItem.new({
                    id: 'cut',
                    text: '剪切',
                    action: () => {
                        doCopy(target, 'cut');
                    },
                }),
            );
        }

        if (canPasteHere) {
            items.push(
                await MenuItem.new({
                    id: 'paste',
                    text: '粘贴',
                    action: () => {
                        doPaste(target || null);
                    },
                }),
            );
        }

        if (target && !target.system) {
            items.push(
                await MenuItem.new({
                    id: 'rename',
                    text: '重命名',
                    action: () => {
                        setRenamingId(target.id);
                    },
                }),
            );
            items.push(
                await MenuItem.new({
                    id: 'delete',
                    text: '删除',
                    action: () => {
                        remove(target);
                    },
                }),
            );
        }

        if (items.length === 0) return;
        const menu = await Menu.new({ items });
        await menu.popup();
    };

    const handleContext = (e: React.MouseEvent, node: Item) => {
        openNativeMenu(e, node);
    };

    const handleRootContext = (e: React.MouseEvent) => {
        openNativeMenu(e, null);
    };

    const createIn = async (parent: Item | null, kind: 'file' | 'folder') => {
        const n = kind === 'file' ? createFile() : createFolder();
        const next = addChild(userList, parent ? parent.id : null, n);
        await apply(next);
        if (kind === 'file') setCurrent(n);
        if (parent && parent.type === 'folder') setExpanded((s) => new Set(s).add(parent.id));
    };

    const rename = async (id: string, name: string) => {
        if (!name) return;
        await apply(updateItemName(userList, id, name));
    };

    const remove = async (node: Item) => {
        if (node.system) return;
        if (await confirm({ title: '删除', description: '确定删除该节点及其所有子节点？' })) {
            await apply(removeItem(userList, node.id));
            if (current?.id === node.id) setCurrent(null);
            await writeHostsToSystem();
        }
    };

    const toggle = async (id: string) => {
        const isOpen = expanded.has(id);
        setExpanded((s) => {
            const ns = new Set(s);
            if (isOpen) ns.delete(id);
            else ns.add(id);
            return ns;
        });
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
            const next = moveNode(
                userList,
                clipboard.node.id,
                targetFolder ? targetFolder.id : null,
            );
            await apply(next);
            setClipboard(null);
        }
    };

    const menuActions = () => {
        return null;
    };

    return (
        <div className={styles.wrapper} onContextMenu={handleRootContext}>
            <div className={styles.treeRoot}>
                {userList.length === 0 && (
                    <div className={styles.emptyHint}>右键空白新建文件或文件夹</div>
                )}
                {userList.map((it: Item) => (
                    <TreeNode
                        key={it.id}
                        node={it}
                        depth={0}
                        activeId={current?.id || null}
                        onSelect={(n) => setCurrent(n.type === 'file' ? n : current)}
                        onToggle={toggle}
                        onRename={rename}
                        onCheck={setOn}
                        expanded={expanded}
                        setExpanded={setExpanded}
                        onContextMenu={handleContext}
                        renamingId={renamingId}
                        setRenamingId={setRenamingId}
                    />
                ))}
            </div>
            {menuActions()}
        </div>
    );
}
