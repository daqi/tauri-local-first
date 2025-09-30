# Implementation Plan: 本地优先多应用智能底座（MCP stdio 交互）

**Branch**: `001-tauri-mcp-stdio` | **Date**: 2025-09-30 | **Spec**: `specs/001-tauri-mcp-stdio/spec.md` **Input**: Feature specification from `./spec.md`

## Execution Flow (/plan command scope)

```
1. Load feature spec from Input path
   → If not found: ERROR "No feature spec at {path}"
2. Fill Technical Context (scan for NEEDS CLARIFICATION)
   → Detect Project Type from file system structure or context (web=frontend+backend, mobile=app+api)
   → Set Structure Decision based on project type
3. Fill the Constitution Check section based on the content of the constitution document.
4. Evaluate Constitution Check section below
   → If violations exist: Document in Complexity Tracking
   → If no justification possible: ERROR "Simplify approach first"
   → Update Progress Tracking: Initial Constitution Check
5. Execute Phase 0 → research.md
   → If NEEDS CLARIFICATION remain: ERROR "Resolve unknowns"
6. Execute Phase 1 → contracts, data-model.md, quickstart.md, agent-specific template file (e.g., `CLAUDE.md` for Claude Code, `.github/copilot-instructions.md` for GitHub Copilot, `GEMINI.md` for Gemini CLI, `QWEN.md` for Qwen Code or `AGENTS.md` for opencode).
7. Re-evaluate Constitution Check section
   → If new violations: Refactor design, return to Phase 1
   → Update Progress Tracking: Post-Design Constitution Check
8. Plan Phase 2 → Describe task generation approach (DO NOT create tasks.md)
9. STOP - Ready for /tasks command
```

**IMPORTANT**: The /plan command STOPS at step 7. Phases 2-4 are executed by other commands:

- Phase 2: /tasks command creates tasks.md
- Phase 3-4: Implementation execution (manual or via tools)

## Summary

该特性为“智能底座”能力：主应用可解析用户的自然语言或显式指令，将其映射为一组结构化动作意图，并通过扫描到的本地子应用描述(`tlfsuite.json`)定位动作并执行（并行受限）。核心目标：可靠、多动作聚合执行、可 Dry Run、可复用解析缓存、对失败/部分成功有清晰反馈。技术路径：

1. 解析层（NLP 轻量规则 + 显式 `app:action(args)` 语法）→ 生成 ParsedIntent[]。
2. 计划层：去重、冲突检测（互斥）、并行批次分组（受并行上限约束）、构建 ExecutionPlan。
3. 调度层：并发执行动作调用（MCP stdio / deeplink / 进程命令）；超时(5s)与合并触发(800ms)处理；Dry Run 直接模拟结果。
4. 结果层：聚合 ActionResult[] → overallStatus（success|partial|failed）。
5. 历史层：持久化 30 日内命令签名与解析摘要，用于复用与审计；自动滚动清理。

本计划 Phase 0/1 仅产出研究、数据模型与契约，不编写实现代码。FR-019（解释模式）暂保留“可选”，将在 research.md 中裁决：若调试价值高则纳入，否则延后。

## Technical Context

**Language/Version**: Rust stable 1.80+ (workspace), TypeScript 5.x (strict, React 18) **Primary Dependencies**: Tauri v2 (IPC/窗口), Radix UI (交互组件), (可能) serde / anyhow / tokio（异步调度），轻量 tokenizer（自实现或简单分词） **Storage**: 本地 SQLite（命令历史 & 缓存解析）；文件系统读取各子应用 `tlfsuite.json` **Testing**: Rust: `cargo test`（单位/集成/契约）；TS: Vitest/Testing Library（解析逻辑、UI 状态） **Target Platform**: 桌面（macOS 首要；Linux/Windows 兼容 Tauri） **Project Type**: 多应用（apps/launcher 主 + apps/\* 子应用）+ 共享 crates & packages **Performance Goals**: 启动额外开销 < 30ms（加载解析与调度组件懒初始化），解析与计划阶段典型复合命令 < 50ms，本地执行批次调度延迟近实时；Dry Run 输出 < 100ms **Constraints**:

- 并行动作上限 = ceil(逻辑核/2) 且 <=4
- 单动作超时 = 5s（不阻塞其他）
- 去抖窗口 = 800ms；历史保留 = 30 天
- 内存附加占用（智能层）常驻 < 10MB（释放缓存策略） **Scale/Scope**: 初期子应用 ~5-8；每次复合命令动作数 <=10（设计硬限制防止 UI 过载）；历史记录 ~ 数百条（30 天）

潜在未知 / 需研究点（进入 Phase 0）：

1. FR-019 解释模式是否影响启动与隐私（展示解析内部权重？）
2. 解析策略：规则+关键词 vs 引入轻量 embedding（权衡体积 vs 模糊匹配质量）
3. 历史签名存储 schema（hash 构成：正则归一化 vs 参数排序）
4. 并发执行在 Tauri 主/子线程与 tokio runtime 的集成方式（避免阻塞 UI）
5. Dry Run 规范：是否需要额外字段区分“预测副作用列表”
6. 冲突检测规则（互斥动作分类标准）
7. 描述文件损坏分类：JSON 解析错误 vs 缺字段 vs 语义冲突（如何分层隔离）

## Constitution Check

_GATE: Must pass before Phase 0 research. Re-check after Phase 1 design._

| Principle | Compliance Assessment | Notes / Planned Safeguards |
| --- | --- | --- |
| Local-First & Offline | PASS | 所有解析/调度/历史均本地；无网络调用；可选在线扩展不在本阶段。 |
| Minimal Footprint | PASS (monitor) | 拒绝引入大型 NLP/embedding 模型；首阶段使用规则+词典；缓存 LRU 控制内存。 |
| Composable Architecture | PASS | 调度/解析抽象为新 crate `crates/intent_core`（命名待定）供 launcher 使用；不嵌入子应用内部逻辑。 |
| Least-Privilege Security | PASS | 仅读取 `tlfsuite.json`；执行动作通过既有受限 IPC / deeplink；不新增文件写入权限。 |
| Testable Modular Reuse | PASS | 计划先定义数据模型 & 契约；解析与计划层拥有独立单元与契约测试；历史持久层抽象可注入。 |
| Additional Constraints (Versioning/Contracts) | PASS | `tlfsuite.json` 字段读取保持向后兼容；缺失字段做 graceful degrade。 |
| Observability (Lightweight) | PASS | 仅结构化调试日志（debug 构建）；Release 默认关闭详细日志。 |
| i18n & Accessibility | PASS (later verify) | 解析结果 UI 使用 Radix，无键盘陷阱；文案集中提取。 |

初步未发现必须记录的违例；若 FR-019 需要暴露解析内部权重且引入较大依赖，将在 Phase 0 重新评估并记录 Complexity Tracking。

## Project Structure

### Documentation (this feature)

```
specs/[###-feature]/
├── plan.md              # This file (/plan command output)
├── research.md          # Phase 0 output (/plan command)
├── data-model.md        # Phase 1 output (/plan command)
├── quickstart.md        # Phase 1 output (/plan command)
├── contracts/           # Phase 1 output (/plan command)
└── tasks.md             # Phase 2 output (/tasks command - NOT created by /plan)
```

### Source Code (repository root)

```
crates/
   intent_core/            # 新增：解析/计划/冲突检测/并行调度核心（Phase 1 后创建）
   intent_history/         # 新增：历史与签名缓存抽象（如与其他模块解耦则独立，否则并入 intent_core）

apps/launcher/
   src/modules/intent/     # 前端 UI：输入框/结果面板/历史视图（TS 解析轻逻辑或调用 Rust via commands）
   src-tauri/src/commands/ # 对外 Tauri commands：parse_intent, execute_plan, dry_run, list_history

packages/ui/
   src/components/intent/  # 复用 UI 组件（ActionList, ExecutionStatus, HistoryItem）

tests/
   intent/                 # Rust 集成/契约测试（descriptor 扫描、计划并发、超时模拟）
```

**Structure Decision**: 采用“多 crate + 主应用模块”方案。解析/调度逻辑放入 `crates/intent_core`，隔离副作用；历史策略若简单则整合在同一 crate 下的 feature module。前端 UI 放在 launcher 下的模块目录并通过 Tauri commands 与核心交互，保持最小 IPC 面。避免把解析逻辑放在前端以减少重复与逆向难度。没有单独“backend”服务（遵循本地优先原则）。

## Phase 0: Outline & Research

1. **Extract unknowns from Technical Context** above:

   - For each NEEDS CLARIFICATION → research task
   - For each dependency → best practices task
   - For each integration → patterns task

2. **Generate and dispatch research agents**:

   ```
   For each unknown in Technical Context:
     Task: "Research {unknown} for {feature context}"
   For each technology choice:
     Task: "Find best practices for {tech} in {domain}"
   ```

3. **Consolidate findings** in `research.md` using format:
   - Decision: [what was chosen]
   - Rationale: [why chosen]
   - Alternatives considered: [what else evaluated]

**Output**: research.md with all NEEDS CLARIFICATION resolved

## Phase 1: Design & Contracts

_Prerequisites: research.md complete_

1. **Extract entities from feature spec** → `data-model.md`:

   - Entity name, fields, relationships
   - Validation rules from requirements
   - State transitions if applicable

2. **Generate API contracts** from functional requirements:

   - For each user action → endpoint
   - Use standard REST/GraphQL patterns
   - Output OpenAPI/GraphQL schema to `/contracts/`

3. **Generate contract tests** from contracts:

   - One test file per endpoint
   - Assert request/response schemas
   - Tests must fail (no implementation yet)

4. **Extract test scenarios** from user stories:

   - Each story → integration test scenario
   - Quickstart test = story validation steps

5. **Update agent file incrementally** (O(1) operation):
   - Run `.specify/scripts/bash/update-agent-context.sh copilot` **IMPORTANT**: Execute it exactly as specified above. Do not add or remove any arguments.
   - If exists: Add only NEW tech from current plan
   - Preserve manual additions between markers
   - Update recent changes (keep last 3)
   - Keep under 150 lines for token efficiency
   - Output to repository root

**Output**: data-model.md, /contracts/\*, failing tests, quickstart.md, agent-specific file

## Phase 2: Task Planning Approach

_This section describes what the /tasks command will do - DO NOT execute during /plan_

**Task Generation Strategy**:

- Load `.specify/templates/tasks-template.md` as base
- Generate tasks from Phase 1 design docs (contracts, data model, quickstart)
- Each contract → contract test task [P]
- Each entity → model creation task [P]
- Each user story → integration test task
- Implementation tasks to make tests pass

**Ordering Strategy**:

- TDD order: Tests before implementation
- Dependency order: Models before services before UI
- Mark [P] for parallel execution (independent files)

**Estimated Output**: 25-30 numbered, ordered tasks in tasks.md

**IMPORTANT**: This phase is executed by the /tasks command, NOT by /plan

## Phase 3+: Future Implementation

_These phases are beyond the scope of the /plan command_

**Phase 3**: Task execution (/tasks command creates tasks.md)  
**Phase 4**: Implementation (execute tasks.md following constitutional principles)  
**Phase 5**: Validation (run tests, execute quickstart.md, performance validation)

## Complexity Tracking

_Fill ONLY if Constitution Check has violations that must be justified_

| Violation                  | Why Needed         | Simpler Alternative Rejected Because |
| -------------------------- | ------------------ | ------------------------------------ |
| [e.g., 4th project]        | [current need]     | [why 3 projects insufficient]        |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient]  |

## Progress Tracking

_This checklist is updated during execution flow_

**Phase Status**:

- [x] Phase 0: Research complete (/plan command)
- [x] Phase 1: Design complete (/plan command)
- [ ] Phase 2: Task planning complete (/plan command - describe approach only)
- [ ] Phase 3: Tasks generated (/tasks command)
- [ ] Phase 4: Implementation complete
- [ ] Phase 5: Validation passed

**Gate Status**:

- [x] Initial Constitution Check: PASS
- [ ] Post-Design Constitution Check: PASS
- [ ] All NEEDS CLARIFICATION resolved (FR-019 待定)
- [ ] Complexity deviations documented

---

_Based on Constitution v1.0.0 - See `/.specify/memory/constitution.md`_
