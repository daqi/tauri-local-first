# Tasks: 本地优先多应用智能底座（MCP stdio 交互）

Feature Dir: `specs/001-tauri-mcp-stdio`  
Spec: `specs/001-tauri-mcp-stdio/spec.md`  
Plan: `specs/001-tauri-mcp-stdio/plan.md`  
Research: `research.md`  
Data Model: `data-model.md`  
Contracts: `contracts/`  
Quickstart: `quickstart.md`

执行策略: TDD 优先，模型/契约 → 测试 → 实现 → 集成 → UI → 性能/清理。标记 [P] 的任务可并行（不同文件/模块）。

## Legend

- [P]: 可并行执行
- Dep: 依赖前序任务完成
- 输出路径需精确创建/修改

---

## Ordered Task List

T001 [X] Setup: 创建新 crate `crates/intent_core` (Cargo.toml, lib.rs) 与初始模块 skeleton；添加到 workspace。Dep: None

T002 [X] Setup: 在 `crates/intent_core` 添加依赖：`serde`, `serde_json`, `thiserror`, `blake3`, `smallvec`, `tokio`（features: sync/rt-multi-thread, time），`anyhow`。Dep: T001

T003 [X] Setup: 在 `apps/launcher/src-tauri/Cargo.toml` 添加对 `intent_core` 的依赖并启用 `serde` features；更新 workspace 根 `Cargo.toml` 确保成员列入。Dep: T002

T004 Setup: 建立 SQLite 访问抽象（若已存在复用），在 `crates/intent_core/src/history/` 建空模块 `mod.rs` + trait `HistoryStore`（内存 stub）。Dep: T003

T005 [X] Model Test [P]: 为数据模型创建 Rust struct 定义 (ParsedIntent, ExecutionPlan, ExecutionPlanBatch, ConflictDetection, ActionResult, CommandHistoryRecord, DescriptorLoadIssue, NormalizeConfig)。仅含字段 + serde derive。位置：`crates/intent_core/src/model.rs`。编写单元测试验证 roundtrip serde。Dep: T002

T006 [X] Model Test [P]: 实现签名归一化函数 `normalize_signature(intents: &[ParsedIntent]) -> String` + 测试：参数顺序 / 动作顺序不同生成相同签名。路径：`crates/intent_core/src/signature.rs`。Dep: T005

T007 [X] Model Test [P]: 实现并发上限计算函数 `compute_concurrency(logical: usize) -> usize` + 测试核心值（2→1,4→2,6→3,8→4,16→4）。路径：`crates/intent_core/src/concurrency.rs`。Dep: T005

T008 [X] Parser Test: 创建解析规则模块 `crates/intent_core/src/parser/`，定义 trait `IntentParser` 与简单规则解析实现（关键词 → 动作）。编写测试：给定样例输入生成期望 intents 列表（含 explicit 语法 `hosts:switch(dev)`）。Dep: T005

T009 [X] Conflict Test: 创建 `conflict` 模块，函数 `detect_conflicts(intents)` 基于 conflictKey 返回 ConflictDetection[]；测试构造互斥动作。Dep: T005

T010 [X] Plan Test: 创建执行计划构建器 `plan::build_plan(intents, max_concurrency)` → 生成去重、冲突、批次；测试去重与批次数量正确。Dep: T006,T007,T009

T011 Timeout Simulation Test: 添加调度 skeleton `executor::execute(plan, opts)`（尚不调用真实子应用），模拟一个动作 sleep 超过 5s 触发 timeout；用 tokio test。Dep: T010

T012 Dry Run Test: 在 executor 增加 dry_run 分支生成 `status=simulated` 与 `predictedEffects`；测试与真实执行结构一致。Dep: T011

T013 Explain Mode Test: 在 parser 返回 ExplainPayload（tokens + matchedRules），编写开启 vs 关闭测试。Dep: T008

T014 History Store Test: 设计 trait HistoryStore 方法：`save(record)`, `list(limit, after)`, `purge_older_than(ts)`；内存实现与测试滚动清理逻辑（>30 天）。Dep: T005

T015 SQLite Adapter: 若选 SQLite，创建 `history/sqlite_store.rs` 使用 `rusqlite`（若未依赖则添加）实现 HistoryStore，其中 purge 在 save 时触发。测试迁移建表。Dep: T014

T016 Descriptor Scan Test: 新模块 `descriptor::scan(root_paths)` 读取 `tlfsuite.json` (mock fs with test fixtures) → ApplicationDescriptor[] + issues; 测试三类错误分类。Dep: T005

T017 Integration Test [P]: 组合 scan + parser + plan + dry_run，验证 overallStatus=success (全 simulated)。路径：`tests/intent/it_dry_run.rs`。Dep: T012,T016

T018 Integration Test [P]: 模拟并发 >4 动作（构造 6 intents），验证批次拆分与限制。路径：`tests/intent/it_concurrency.rs`。Dep: T010

T019 Integration Test [P]: 模拟一个动作挂起 + 其它快完成，确认 timeout 不阻塞其它完成，overallStatus=partial。`tests/intent/it_timeout.rs`。Dep: T011

T020 Integration Test [P]: 冲突检测案例（两个互斥 hosts 切换）→ 计划含 conflicts 并标记策略 `force-order`。`tests/intent/it_conflict.rs`。Dep: T010

T021 Integration Test [P]: 历史复用：前后两次相同输入签名一致且第二次解析阶段可标记 cache hit (添加 flag)。`tests/intent/it_history_cache.rs`。Dep: T014,T010

T022 Tauri Command Skeleton: 在 `apps/launcher/src-tauri/src/commands/intent.rs` 创建 commands: parse_intent, dry_run, execute_plan, list_history（目前 stub 返回 NotImplemented 错误）。更新 `tauri.conf.json` capability 如需。Dep: T012,T014,T010

T023 Command Contract Tests: 使用 Rust integration (or TS if easier) 测试 parse_intent stub 接口形状（断言错误码 NOT_IMPLEMENTED）。Dep: T022

T024 Implement parse_intent: 调用 parser + plan 构建（不执行），支持 explain。移除 stub。测试通过。Dep: T008,T010,T013

T025 Implement dry_run: 复用 parse 流程 + executor(dry_run) + history 保存（记录 explainUsed）。Dep: T012,T014,T024

T026 Implement execute_plan (input 分支): 与 dry_run 类似但真实执行（当前仍为 mock executor），保存历史。Dep: T011,T014,T024

T027 Implement execute_plan (planId 分支): 支持缓存计划（内存 map）；添加过期策略（短期 e.g. 2 分钟）。Dep: T026

T028 Implement list_history: 查询 HistoryStore + 分页; 测试 after/limit。Dep: T014,T022

T029 Real Executor Hook: 在 executor 中替换 mock 调用为抽象 `ActionInvoker` trait；暂留默认 no-op 或 logging。Dep: T026

T030 UI Components [P]: 在 `packages/ui/src/components/intent/` 创建：ActionList.tsx, ExecutionStatus.tsx, HistoryList.tsx（基于 Radix primitives）。Dep: T022

T031 Launcher Module Wiring: 在 `apps/launcher/src/modules/intent/` 添加 hooks + store（Zustand/Recoil? 若无则简单 context），调用 Tauri commands 更新 UI。Dep: T024,T025,T026,T028,T030

T032 UI Integration Test: (Vitest + jsdom) 模拟输入 → mock command 返回 → 渲染执行结果与 history。Dep: T031

T033 Performance Budget Checks: 添加简单 bench（criterion or timing test）验证 normalize_signature 与 plan 构建 < X ms （阈值 5ms/n=10 intents）。Dep: T006,T010

T034 Cleaning & Security Pass: 审核不必要依赖，确认无网络调用；输入参数校验（长度、空值）补测试。Dep: T031,T028

T035 Documentation Update [P]: 更新 `docs/CONVENTIONS.md` 增加 descriptor 错误分类 & explain 模式说明；在 README 主项目加入 ✅ 进度标记。Dep: T029

T036 Final Quickstart Validation: 按 `quickstart.md` 手动/脚本跑通；记录任何偏差生成 issue 列表。Dep: T032,T028,T026

T037 Post-Design Constitution Check: 复核最小依赖 & 离线；若引入 rusqlite 等补 justification 注释。Dep: T036

T038 Ready for Implementation Closure: 汇总未完成的低优先升级点（embedding 语义、真实 ActionInvoker）。Dep: T037

T039 Debounce Merge Test: 针对 FR-011 实现去抖合并逻辑（记录 merged 状态）。编写测试：800ms 内两次相同输入只执行一次计划，第二次返回 merged 引用。位置：`tests/intent/it_debounce_merge.rs`. Dep: T010,T014

T040 Availability & Permission Validation: 针对 FR-003 添加可用性校验模块 `validator/`（检测 descriptor 有效与 requiresElevation 标记），并编写失败/缺权限 mock 测试。Dep: T016

T041 Partial Success & Retry Hint Test: 构造一成功一失败动作（requiresElevation 模拟），验证聚合结果 overallStatus=partial 且失败 ActionResult.retryHint 存在。Dep: T040,T011

T042 MCP Stdio JSON Roundtrip Test: 最小 stdio 适配层 stub（以纯文本行读取/写入 JSON）+ 测试保证无多余前后缀。Dep: T024

T043 Conflict Resolution Behavior: 测试当 resolution=force-order 时执行顺序符合计划；当需要 user-select 时返回 PLAN_CONFLICT_UNRESOLVED 错误码。Dep: T020,T024

T044 Cache Revalidation Test: 两次相同输入，第二次前模拟禁用一个动作（descriptor 失效）→ 计划重新标记不可用并不执行该动作。Dep: T021,T040

T045 Explain Default Off Test: 未传 explain 选项 parse_intent 不返回 explain 字段；传 true 返回。Dep: T013

T046 Standard Result Object Test: 针对 FR-020 验证 fields = overallStatus + actions[*].{intentId,status,reason?,retryHint?,predictedEffects?}；部分成功 case 覆盖。Dep: T041,T012

T047 Input Validation Negative Tests: 针对 parse_intent/dry_run/execute_plan/list_history：空 input、planId+input 同时提供、limit<0、after<0，返回统一错误码。Dep: T022

T048 Dependency Justification Note: 添加/更新 README 或 PR 模板片段，记录 rusqlite 与 blake3 依赖 rationale（最小足迹说明），保证宪章最小依赖合规。Dep: T015

T049 UI No Direct Parse Guarantee: 添加 UI 测试/代码检查，确保前端模块不直接构造 ParsedIntent（只能通过 command 响应），若尝试则测试失败。Dep: T031

T050 LRU Cache Policy Definition: 实现解析缓存 LRU (上限 100 entries 或 30 分钟 TTL) + 单元测试驱逐最旧条目。Dep: T010

T051 History Purge On Save Test: 模拟插入超期记录后 save 新记录触发 purge；验证数据库记录数下降。Dep: T015

T052 Startup Overhead Measurement: 添加基准脚本/测试测量初始化（加载 parser + plan builder）耗时 <30ms (Release or mocked). Dep: T033

T053 Update Spec Alignment: Patch spec FR-020 示例与 data-model 字段对齐（intentId + predictedEffects）。(文档任务) Dep: T046

T054 Constitution Post-Remediation Check: 复核新增任务满足 Least-Privilege & Minimal Footprint；若通过标记完成。Dep: T053

## Parallel Guidance

- Model 构建相关 (T005,T006,T007) 可并行。
- Integration tests 组 (T017–T021) 可在核心计划/执行测试完成后并行运行。
- UI 组件 (T030) 可与命令实现后期并行，只要命令 skeleton 已存在。

## Agent Command Examples

执行并行示例（概念）:

```
run_task T005 & run_task T006 & run_task T007
```

（实际由调度器/人工按 [P] 标记并行）

## Exit Criteria

- 所有契约命令具备实现并通过测试
- Quickstart 全流程无阻塞
- 性能基准满足预算
- 宪章再核对无新增违例

-- END --
