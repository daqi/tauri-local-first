# 路线图（Roadmap）

> 管理套件化本地优先应用的规划、里程碑与状态。状态使用：🔜 计划｜🚧 开发中｜✅ 可用｜⏸️ 暂停

## 目标
- 打造可复用的主应用（Launcher）与插件化架构，统一托盘、快捷键、设置、更新。
- 以 React + Radix UI + Tauri v2 为主栈，保持体积小、低占用、默认离线。
- 通过 deeplink（`tlfsuite://`）与命令参数实现 app 间互操作。

## 里程碑

### M1：主应用 MVP（🚧）
- [ ] 启动器界面（Radix UI，命令面板占位）
- [ ] 事件总线与命令调用（`open_with_args`）
- [ ] 深链占位：`tlfsuite://open?app=hosts&args=...`
- [ ] 托盘与全局快捷键基础（可选）
- [ ] 构建与签名流程占位（CI 准备）

### M2：Hosts 管理器对接（🚧）
- [x] 规则启用/停用
- [ ] 分组切换
- [ ] 备份/还原
- [ ] 预览与冲突检测（高）
- [ ] 历史版本/回滚（带差异对比）（高）
- [ ] 刷新 DNS（可选，需管理员权限）
- [ ] 导入/导出

### M3：剪贴板管家 MVP（🔜）
- [ ] 托盘与全局快捷键
- [ ] 文本/图片持久化（SQLite）
- [ ] 搜索与去重
- [ ] 设置页与导出

### M4：基础设施（🔜）
- [ ] `packages/ui` 完善（封装 Radix 常用组件）
- [ ] `crates/core`（配置、日志、校验）
- [ ] `crates/system`（Clipboard/Hosts/FS 封装）
- [ ] `crates/jobs`（并发任务队列）

## 规范与实践
- 脚本与包管理使用 `pnpm`，工作区在根目录。
- Tauri `beforeDevCommand=pnpm dev` / `beforeBuildCommand=pnpm build`（或根脚本）。
- 文档状态仅用 emoji，不写日期。

## 链接
- 主 README：`../README.md`
- Hosts 管理器仓库：https://github.com/daqi/SweetHosts
