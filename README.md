# 本地优先 · Tauri 替代 Electron 计划

> 用 Tauri + Rust 打造一组常用小工具，体积更小、占用更低、完全本地可用；沉淀可复用的多应用底座。

- 文档：参见 [路线图（Roadmap）](./docs/ROADMAP.md)

## 状态标识
- 🔜 计划
- 🚧 开发中
- ✅ 可用
- ⏸️ 暂停（可选）

## 当前聚焦
- 剪贴板管家 🚧
- Hosts 管理器 🚧
- 候选排队：本地笔记、图片批量压缩

## 应用清单与进度
- 剪贴板管家（历史/去重/模糊搜索/全局快捷键/托盘） 🚧
  - [ ] 托盘与全局快捷键
  - [ ] 文本/图片记录与本地持久化（SQLite）
  - [ ] 搜索、去重、粘贴增强
  - [ ] 设置页与数据导出
- 本地笔记（Markdown + 双链/标签 + FTS5 搜索） 🔜
  - [ ] 笔记库与附件目录
  - [ ] Markdown 编辑与预览
  - [ ] FTS5 搜索与反向链接
  - [ ] 导入/导出
- 图片批量压缩/转换（PNG/JPEG/WebP） 🔜
  - [ ] 拖放批处理
  - [ ] 并发任务队列
  - [ ] 质量预设与预览
  - [ ] 元数据处理（EXIF 可选保留）
- 截图与离线 OCR（Tesseract） 🔜
  - [ ] 区域截图
  - [ ] 本地 OCR
  - [ ] 一键复制/保存
- PDF 工具箱（合并/拆分/提取/压缩） 🔜
- JSON/CSV 工具箱（校验/格式化/转换/大文件） 🔜
- Hosts 管理器（分组切换/规则启用、备份/还原） 🚧
  - [x] 规则启用/停用
  - [ ] 分组切换
  - [ ] 备份/还原
  - [ ] 预览与冲突检测（高）
  - [ ] 历史版本/回滚（带差异对比）（高）
  - [ ] 刷新 DNS（可选，需管理员权限）
  - [ ] 导入/导出
- 快速启动器（全局搜索/命令/工作流、可扩展插件） 🔜
- 轻量 API 客户端（环境/脚本/历史/本地加密） 🔜
- 番茄钟 + 时间追踪（托盘/统计/导出） 🔜

## 工程与底座（简要）
- 技术栈：Tauri v2 · Rust · React + Radix UI · SQLite（FTS5）。
- 常用能力：FS/Path/Dialog/HTTP/Store/Tray/Global Shortcut/Window State/Autostart/Updater。
- 复用形态：单仓多应用（pnpm workspace + Cargo workspace），共享 UI 组件与 Rust 工具库。
- 安全与体积：本地资源、命令白名单、最小权限、懒加载、侧车按需启用。
- 质量与发布：TypeScript/Rust 全量类型；ESLint/Prettier/Clippy；Changesets；CI 三平台构建。

### 依赖选型说明（最小足迹合规）

仅列出需架构级理由的核心底座依赖：

| 依赖 | 用途 | 选择理由 | 拒绝的替代方案 | 宪章一致性 |
| ---- | ---- | -------- | -------------- | ---------- |
| `rusqlite` (bundled) | 持久化（历史/配置/后续 FTS5） | 单文件嵌入式 DB，零守护进程；成熟稳定；启用 `bundled` 便于可移植构建 | `sled`（缺少成熟 SQL/FTS）、外部 PostgreSQL（增加部署复杂度） | Local‑First / 最小运维 |
| `blake3` | 意图签名哈希（归一化后） | 极高吞吐 + 可并行；实现体积小；比 SHA256 更快且足够低碰撞 | `sha2`（较慢，超出需求），自定义 murmur/fnv（碰撞率/实现负担） | 性能 / 最小依赖 |

设计约束：
- 不引入除标准库 + 上述必要 crates 外的重型运行时或服务进程。
- 若后续需要全文检索，将复用同一 SQLite (FTS5) 而非新增搜索后端。
- 哈希用途为本地缓存键（非安全敏感），`blake3` 已满足速度与可靠性，无需更重加密套件。

变更评估策略：当出现 (a) 解析/历史 写入性能瓶颈 或 (b) 哈希碰撞率观测 > 0.1% 时再评估替代方案。


### 应用发现与描述 (Descriptor)

每个可被 Launcher 发现的应用需提供 `tlfsuite.json`，放置位置（按优先匹配顺序）：

1. `<appRoot>/tlfsuite.json`
2. `<appRoot>/Contents/Resources/tlfsuite.json` (macOS .app bundle)
3. `<appRoot>/resources/tlfsuite.json`
4. `<appRoot>/share/tlfsuite/tlfsuite.json`

打包（Tauri）时在 `tauri.conf.json` 的 `bundle.resources` 中包含：

```jsonc
{
  "bundle": { "resources": ["tlfsuite.json"] }
}
```

`tlfsuite.json` 示例：

```jsonc
{
  "id": "hosts",
  "name": "Hosts Manager",
  "description": "Manage /etc/hosts rules",
  "scheme": "hostsmanager", // 可选，提供则 deep link 用 <scheme>://open
  "actions": [
    { "name": "open", "title": "Open" },
    { "name": "rule", "title": "Open Rule", "args": [{ "name": "id", "type": "string", "required": true }] }
  ],
  "icon": "icon.png" // 可为 data:URL / 相对路径 / (Linux) 图标名称
}
```

### Deep Link 构造

优先使用应用自定义 scheme：

```
<scheme>://open?args=<urlencoded>
```

否则回退统一入口：

```
tlfsuite://open?app=<id>&args=<urlencoded>
```

## 原则
- 本地优先、默认可离线
- 零配置即用、按需启用
- 小体积、低占用、可维护

## 参与与反馈
- 欢迎通过 Issue/Discussion 提需求或投票决定优先级。
- 标记为 `good first issue` 的任务适合首次贡献。
- 不设具体日期，按上方“状态标识”推进各应用阶段。
