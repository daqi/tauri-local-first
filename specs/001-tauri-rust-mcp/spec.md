# Feature Specification: Multi‑App Local-First Suite Descriptor & Inter-App Invocation

**Feature Branch**: `001-tauri-rust-mcp`  
**Created**: 2025-09-29  
**Status**: Draft  
**Input**: User description: "我正在用 Tauri + Rust 打造一组常用小工具，体积更小、占用更低、完全本地可用，各应用之间可以实现互相调用，交互基于类似MCP的描述文件，其他信息查看项目里的文档"

## Execution Flow (main)
```
1. Parse user description from Input
   → If empty: ERROR "No feature description provided"
2. Extract key concepts from description
   → Identify: actors, actions, data, constraints
3. For each unclear aspect:
   → Mark with [NEEDS CLARIFICATION: specific question]
4. Fill User Scenarios & Testing section
   → If no clear user flow: ERROR "Cannot determine user scenarios"
5. Generate Functional Requirements
   → Each requirement must be testable
   → Mark ambiguous requirements
6. Identify Key Entities (if data involved)
7. Run Review Checklist
   → If any [NEEDS CLARIFICATION]: WARN "Spec has uncertainties"
   → If implementation details found: ERROR "Remove tech details"
8. Return: SUCCESS (spec ready for planning)
```

---

## ⚡ Quick Guidelines
- ✅ Focus on WHAT users need and WHY
- ❌ Avoid HOW to implement (no tech stack, APIs, code structure)
- 👥 Written for business stakeholders, not developers

### Section Requirements
- **Mandatory sections**: Must be completed for every feature
- **Optional sections**: Include only when relevant to the feature
- When a section doesn't apply, remove it entirely (don't leave as "N/A")

### For AI Generation
When creating this spec from a user prompt:
1. **Mark all ambiguities**: Use [NEEDS CLARIFICATION: specific question] for any assumption you'd need to make
2. **Don't guess**: If the prompt doesn't specify something (e.g., "login system" without auth method), mark it
3. **Think like a tester**: Every vague requirement should fail the "testable and unambiguous" checklist item
4. **Common underspecified areas**:
   - User types and permissions
   - Data retention/deletion policies  
   - Performance targets and scale
   - Error handling behaviors
   - Integration requirements
   - Security/compliance needs

---

## User Scenarios & Testing *(mandatory)*

### Primary User Story
作为用户，我安装（或仅下载解压）该套件中的一个或多个桌面小工具应用（如 Hosts 管理器、剪贴板管家）。我希望：
1. 每个应用独立运行、默认离线且资源占用低；
2. Launcher 或其他应用可以发现并调用这些应用暴露的动作（actions），例如让 Hosts 管理器切换到某个规则组，或从剪贴板管家获取最近复制的文本；
3. 这种调用无需复杂配置，只依赖每个应用随构建产物一同发布的描述文件（Descriptor，类似 MCP 协议的“能力声明”）；
4. 我可以通过 Deep Link、命令面板、或统一命令路由的方式跨应用执行动作，并在被调用应用中获得明确结果（成功、失败、参数错误）。

### Acceptance Scenarios
1. **Given** 已安装两个支持描述文件的应用 (A=Hosts, B=Clipboard)，**When** Launcher 扫描本地约定路径 (可执行旁/tlfsuite.json) **Then** 列表中展示 A 与 B 的名称、图标、可用 actions。
2. **Given** Hosts 应用描述文件中声明 action `switch_group` 需要参数 `groupId` (string, required)，**When** 用户通过 Launcher 输入命令 `hosts switch_group --groupId=dev` 并执行 **Then** Hosts 应用收到请求并返回“切换成功”反馈且组状态更新可在 Hosts UI 中看到。
3. **Given** 用户通过 Deep Link `hostsmanager://open?args=...` 调起应用 **When** 应用已经在运行 **Then** 应用前置并处理该 action（幂等）。
4. **Given** 用户传入缺失必填参数的 action 调用 **When** 调用被解析 **Then** 调用方获得结构化错误（含缺失参数名），不会使目标应用崩溃。
5. **Given** 某应用未暴露任何 action（仅基础界面）**When** Launcher 扫描 **Then** 该应用仅显示“可启动”而无可执行动作列表。
6. **Given** 两个应用声明相同 action 名称但不同 `id` **When** 用户通过 `tlfsuite://open?app=<id>&args=...` 指定 app **Then** 正确路由到对应应用，不产生冲突。
7. **Given** 用户离线且未连接网络 **When** 执行跨应用 action **Then** 行为成功（不依赖远端）且耗时低于 500ms（若目标 app 已启动）。

### Edge Cases
- 同一 action 被快速重复触发（双击、脚本循环） → 需保证幂等或返回“正在处理中”。
- 描述文件损坏 / JSON 解析失败 → Launcher 跳过该应用并记录错误（不阻塞其他应用发现）。
- Action 参数类型与调用方不匹配（数字传入字符串等） → 返回结构化校验错误。
- Deep Link 在目标应用尚未安装或未在 PATH 中 → 系统层面无响应或调用方需提示“未找到应用”。
- 用户尝试调用未公开（未在 descriptor 中列出）的内部功能 → 拒绝并返回“action not found”。

## Requirements *(mandatory)*

### Functional Requirements
- **FR-001**: 套件 MUST 支持通过扫描约定路径发现已安装应用的描述文件 (`tlfsuite.json`) 并建立内存索引。
- **FR-002**: 每个描述文件 MUST 至少包含 `id`, `name`, `description`, `actions[]`（可为空数组），可选包含 `scheme`, `icon`。
- **FR-003**: 系统 MUST 校验描述文件结构；若关键字段缺失或类型错误则跳过该应用并产生日志。
- **FR-004**: Launcher MUST 能列出所有被成功解析的应用及其 actions，供搜索/过滤。
- **FR-005**: 调用方（Launcher 或其他应用） MUST 能以统一调用模型触发 action：
   - a) 通过应用自定义 scheme Deep Link（若存在）
   - b) 通过统一入口 scheme `tlfsuite://open?app=<id>&args=<encoded>`
   - c) 通过本地专用轻量 IPC 通道（单机内：基于命名管道/本地 socket/事件桥抽象；对业务只暴露“发送请求+接收结果”接口），用于避免频繁 Deep Link 带来的窗口聚焦与参数长度限制。
- **FR-006**: 被调用应用 MUST 校验 action 名称与参数并返回结构化结果（成功/错误/处理中），错误包含 `code` 与 `message`。
- **FR-007**: 系统 MUST 支持 action 参数的基本类型声明：`string|number|boolean` 以及 `required` 标记。
- **FR-008**: 系统 MUST 在调用前进行参数本地验证；当验证失败不触发目标应用。
- **FR-009**: 系统 SHOULD 在同一应用多次快速 action 调用时防抖或标记为重复（幂等性策略由目标应用声明或默认幂等）。
- **FR-010**: 描述文件更新（文件时间戳变化）后 MUST 支持热刷新（不需要重启 Launcher）。
- **FR-011**: 系统 MUST 在离线状态下完整工作，不产生网络请求。
- **FR-012**: 系统 MUST 限制单次扫描耗时 < 200ms（在 ≤10 个应用场景）。
- **FR-013**: 系统 MUST 允许通过命令行参数启动 Launcher 并直接执行一次 action (`--action <app>:<action> --args ...`)。
- **FR-014**: 系统 SHOULD 允许应用声明图标多种来源（data URL / 相对路径）。
- **FR-015**: 日志 MUST 不含敏感本地路径外的用户数据；错误说明聚焦可诊断性。
- **FR-016**: 如果用户调用不存在的应用或 action MUST 返回 `NOT_FOUND` 类型错误。
- **FR-017**: 系统 SHOULD 允许为 action 定义一个简短 `title` 与可选 `args` 描述以便 UI 命令面板展示。
- **FR-018**: 调用返回对象 MUST 最多包含：`status`(success|error|processing), `payload`(可选), `error`(可选), `meta`(耗时)。
- **FR-019**: 系统 MUST 以插件数可线性扩展方式运行（新增 app 不需要修改核心代码，只需 descriptor）。
- **FR-020**: 系统 SHOULD 支持基本冲突检测：若两个应用 `id` 相同，后解析的被忽略并记录冲突。
- **FR-021**: 系统 SHOULD 支持 descriptor 缓存以减少重复 IO（启动后只在变更检测时重新读取）。
- **FR-022**: 系统 MUST 提供最小必要权限（不要求管理员除非应用自身功能需要）。
- **FR-023**: Descriptor MUST 含 `version` 字段（语义化主.次.补丁），用于表示描述文件结构版本（而非应用功能版本）。
- **FR-024**: 系统 MUST 在解析时校验 `version` 主版本与当前支持范围兼容，否则忽略该应用并记录“UNSUPPORTED_DESCRIPTOR_VERSION”。
- **FR-025**: 系统 SHOULD 允许 future 扩展：action 分类、权限要求、兼容范围 (`engines` / `minLauncherVersion`)。

### Key Entities *(include if feature involves data)*
- **Descriptor (tlfsuite.json)**: 抽象一个可调用应用的自描述文档；字段（概念层面）`id`, `name`, `description`, `version`, `scheme?`, `actions[]`, `icon?`，未来可扩展 `categories[]`, `permissions[]`, `engines`。
- **Action**: 可被触发的原子能力；属性：`name`, `title?`, `args[]` (每个 arg: `name`, `type`, `required?`).
- **Invocation Request**: 一次动作调用的输入；属性：`targetAppId`, `actionName`, `args(key-value)`, `timestamp`。
- **Invocation Result**: 调用输出；属性：`status`, `payload?`, `error?{code,message}`, `meta{durationMs}`。
- **Registry / Index**: 内存中的已发现应用与 actions 映射，用于快速检索与校验。
 - **IPC Channel**: 跨应用本地进程通信抽象；提供请求/响应模式，特性：本机可达、低延迟、单一消费者路由、不会触发窗口聚焦。

---

## Review & Acceptance Checklist
*GATE: Automated checks run during main() execution*

### Content Quality
- [ ] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

### Requirement Completeness
- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous  
- [x] Success criteria are measurable
- [x] Scope is clearly bounded
- [ ] Dependencies and assumptions identified

---

## Execution Status
*Updated by main() during processing*

- [ ] User description parsed
- [ ] Key concepts extracted
- [ ] Ambiguities marked
- [ ] User scenarios defined
- [ ] Requirements generated
- [ ] Entities identified
- [ ] Review checklist passed

---
