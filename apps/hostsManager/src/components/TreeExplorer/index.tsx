import { useState } from 'react';
import { useStore } from '../../store';
import styles from './index.module.less';
import { Item } from '../../typing';
// 状态展示不再使用交互式复选框，启用/停用移到右键菜单
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
import { confirm, Tree, type TreeDataItem } from '@suite/ui';
import { SYSTEM_HOST_ITEM } from '@/constants';
import { DotFilledIcon } from '@radix-ui/react-icons';
import { Menu, MenuItem } from '@tauri-apps/api/menu';

interface ClipboardState {
    mode: 'copy' | 'cut';
    node: Item;
}

export default function TreeExplorer() {
    // store: userList (tree), current selected file item, mutateList for immutable updates
    const { userList, current, setCurrent, mutateList } = useStore() as any; // cast for extended fields
    const [clipboard, setClipboard] = useState<ClipboardState | null>(null);

    const apply = async (next: Item[]) => {
        await mutateList(() => next);
    };

    const openNativeMenu = async (e: React.MouseEvent, target: Item | null) => {
        e.preventDefault();
        e.stopPropagation();
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
            items.push(
                await MenuItem.new({
                    id: 'toggle_on',
                    text: target.on ? '停用' : '启用',
                    action: () => {
                        setOn(target.id, !target.on);
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
                        const n = prompt('输入新名称', target.name)?.trim();
                        if (n) rename(target.id, n);
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

    const handleRootContext = (e: React.MouseEvent) => openNativeMenu(e, null);

    const createIn = async (parent: Item | null, kind: 'file' | 'folder') => {
        const n = kind === 'file' ? createFile() : createFolder();
        const next = addChild(userList, parent ? parent.id : null, n);
        await apply(next);
        if (kind === 'file') setCurrent(n);
    // TreeView 自身展开控制: 目前依赖默认行为 (Radix Accordion state 自动管理)
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

    // map to TreeDataItem (system hosts is injected at top, read-only)
    const buildTree = (items: Item[], { includeSystem = false } = {}): TreeDataItem[] => {
        const arr: TreeDataItem[] = [];
        if (includeSystem) {
            arr.push({
                id: SYSTEM_HOST_ITEM.id,
                name: 'System Hosts',
                onClick: () => setCurrent(SYSTEM_HOST_ITEM),
                draggable: false,
                droppable: false,
            });
        }
        items.map((it) => {
            const actions = (
                <div
                    onClick={(e) => e.stopPropagation()}
                    onContextMenu={(e) => openNativeMenu(e, it)}
                    style={{ display: 'flex', alignItems: 'center', gap: 4, fontSize: 12, opacity: 0.9 }}
                    title={it.on ? '已启用' : '已停用'}
                >
                    <DotFilledIcon
                        style={{
                            width: 12,
                            height: 12,
                            color: it.on ? '#10b981' : '#9ca3af',
                            filter: it.on ? 'drop-shadow(0 0 2px #10b98155)' : 'none'
                        }}
                    />
                </div>
            );
            const node: TreeDataItem = {
                id: it.id,
                name: it.name + (it.type === 'folder' ? '' : ''),
                children: it.type === 'folder' && it.children ? buildTree(it.children) : undefined,
                actions,
                onClick: () => {
                    if (it.type === 'file') setCurrent(it);
                },
                draggable: false, // TODO: enable drag & drop with moveNode
                droppable: it.type === 'folder',
            };
            arr.push(node);
        });
        return arr;
    };

    const treeData: TreeDataItem[] = buildTree(userList, { includeSystem: true });

    return (
        <div className={styles.wrapper} onContextMenu={handleRootContext}>
            <Tree
                data={treeData}
                onSelectChange={(ti) => {
                    if (!ti) return;
                    // selection already handled in onClick but keep for safety
                    if (ti.id === SYSTEM_HOST_ITEM.id) {
                        setCurrent(SYSTEM_HOST_ITEM);
                        return;
                    }
                    const found = userList.find((i: Item) => i.id === ti.id);
                    if (found && found.type === 'file') setCurrent(found);
                }}
                onItemContextMenu={(e, ti) => {
                    if (ti.id === SYSTEM_HOST_ITEM.id) return; // no context menu for system
                    const found = userList.find((i: Item) => i.id === ti.id) || null;
                    openNativeMenu(e as any, found);
                }}
            />
        </div>
    );
}
