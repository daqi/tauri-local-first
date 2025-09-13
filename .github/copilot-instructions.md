<!-- Use this file to provide workspace-specific custom instructions to Copilot. For more details, visit https://code.visualstudio.com/docs/copilot/copilot-customization#_use-a-githubcopilotinstructionsmd-file -->

# 本仓库生成代码的约定（务必遵循）
- 项目主题：本地优先的 Tauri 应用套件，替代常见 Electron 工具。
- 首选技术：Rust + Tauri v2；前端 React + Radix UI；构建与包管理使用 pnpm + Vite。
- 架构：单仓（pnpm workspace），目录为 apps/*、packages/*、crates/*、docs/*、scripts/*。
- 目标：最小权限、体积小、内存占用低；API 全量类型；IPC/命令按需、懒加载。
- UX：单窗口为主，托盘与全局快捷键按需；命令面板/深链/命令行参数作为 app 间交互。
- 网络：默认离线；同步/联网是可选并需显式开启。

# 代码风格与实践
- TypeScript 严格模式，React 18，Radix UI 主题组件；优先函数组件 + hooks。
- Rust 使用 Cargo workspace；核心能力抽到 crates（core/system/jobs/storage 等）。
- 所有脚本与安装命令优先使用 pnpm（非 npm/yarn）。
- 生成 Tauri 配置时，beforeDevCommand/dev 与 beforeBuildCommand/build。
- 对外暴露命令需最小化并进行输入校验；敏感操作（如写 hosts）必须二次确认。

# 目录结构（约定）
- apps/launcher：主应用（统一入口、托盘、命令面板、插件注册）。
- apps/<feature>：各功能子应用（clipboard、hosts、ocr、img-tools、pdf-tools、api-client、timer 等）。
- packages/ui：前端 UI 复用包（封装 Radix primitives）。
- crates/<name>：Rust 复用库（配置、系统调用、任务队列、存储等）。

# 任务建议
- 新建应用：使用 React + Radix + Vite + Tauri v2 模板；脚本为 pnpm dev/build + tauri。
- 交互：优先使用 deeplink（tlfsuite://）、命令参数、事件总线三种方式之一。
- 文档：README 里使用 emoji 状态（🔜/🚧/✅/⏸️），不写具体日期。
