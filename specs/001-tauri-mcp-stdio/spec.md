# Feature Specification: 本地优先多应用智能底座（MCP stdio 交互）

**Feature Branch**: `001-tauri-mcp-stdio`  
**Created**: 2025-09-30  
**Status**: Draft  
**Input**: User description: "我正在用 Tauri 打造一组本地优先的常用小应用集，应用交互遵循 MCP 本地 stdio 规范，主应用作为智能底座可以理解人类意图，扫描和调用子应用"

## Clarifications

### Session 2025-09-30

- Q: 最大并行执行动作数上限应是多少？ → A: 动态：CPU 逻辑核心数的一半向上取整 (max 4)
- Q: 命令历史保留时长？ → A: 最近 30 天
- Q: 单个动作调用超时时间？ → A: 5 秒
- Q: 复合命令去抖时间窗口？ → A: 800 ms
- Q: 是否支持 Dry Run 模式？ → A: 是，提供 `--dry-run` / 显式前缀 支持

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

## User Scenarios & Testing _(mandatory)_

### Primary User Story

一个用户启动主应用（智能底座），通过自然语言输入或快捷召唤命令面板，输入“启用开发环境 hosts 规则并打开剪贴板历史”，主应用理解意图：

1. 解析出两个动作（切换 hosts 分组；打开剪贴板应用的历史视图）。
2. 扫描已安装的子应用与其 `tlfsuite.json` 描述，定位对应可执行动作。
3. 顺序或并行调用子应用（本地进程 / deeplink / MCP stdio 会话）。
4. 汇总执行结果并在统一界面返回（成功/失败/部分失败）。

### Acceptance Scenarios

1. **Given** 主应用已完成应用扫描且 hosts 与剪贴板子应用存在，**When** 用户输入含两个意图的自然语言指令，**Then** 系统 MUST 拆分并分别调用两个子应用并反馈整体结果状态（成功/失败明细）。
2. **Given** 用户输入指令引用一个不存在的子应用操作，**When** 系统尝试匹配，**Then** 系统 MUST 返回“未找到动作”且不执行其他不确定操作。
3. **Given** 子应用执行其中一个动作失败（例如 hosts 分组切换权限不足），**When** 另一个动作成功，**Then** 系统 MUST 标记部分成功并包含可重试信息。
4. **Given** 用户第二次输入与上一次高度相似的复合命令，**When** 系统识别已缓存的解析，**Then** 系统 SHOULD 复用解析加速执行并仍然验证子应用可用性。

### Edge Cases

- 用户输入完全模糊，不含已知动作关键词 → 返回澄清问题列表而非空反馈。
- 同一子应用多个潜在动作匹配（歧义）→ 要求用户选择（列表化）。
- 某子应用描述文件缺失或损坏 → 标记该应用“不可调用”并在结果集中附错误原因。
- 子应用长时间无响应（超时）→ MUST 标记该动作失败并继续其他动作（若独立）。
- 用户快速连续触发相同复合命令 → 系统 MUST 合并并序列化（防止重复副作用）。

## Requirements _(mandatory)_

### Functional Requirements

- **FR-001**: 系统 MUST 扫描并缓存本地子应用描述 (`tlfsuite.json`) 列表与动作集合。
- **FR-002**: 系统 MUST 支持将自然语言输入解析为 ≥1 个结构化“意图动作”对象（动词 + 目标 + 参数）。
- **FR-003**: 系统 MUST 校验每个意图映射的子应用与动作是否存在且处于可用状态（含权限前置条件）。
- **FR-004**: 系统 MUST 依序或并行执行多个动作，并收集独立结果（成功/失败/跳过/超时）。
- **FR-005**: 系统 MUST 在单一结果视图中聚合展示全部动作执行状态与可重试指引。
- **FR-006**: 系统 MUST 在动作执行前进行去重（相同目标+参数重复出现时仅执行一次）。
- **FR-007**: 系统 MUST 对超出最大并行数的动作队列化；最大并行数 = `ceil(logical_cpu_cores / 2)` ，且不超过 4（示例：2C→1，4C→2，6C→3，8C+→4）。
- **FR-008**: 系统 MUST 对无法解析的输入返回至少一个澄清性问题（而非静默失败）。
- **FR-009**: 系统 MUST 记录每次复合命令解析（意图集合 + 时间 + 结果摘要）以支持复用与审计，并保留最近 30 天记录；超期记录 MUST 自动清理（滚动删除）。
- **FR-010**: 系统 MUST 在子应用无响应达到 5 秒超时阈值后标记该动作为 timeout 并继续其他未完成动作；若动作声明 `requiresElevation` 则可在失败结果中附加重试提示。
- **FR-011**: 系统 MUST 防止同一复合命令在短时间内被重复并行执行；判定语义等价且 800ms 内再次触发时合并为单次执行（追加为“merged”状态记录），并返回统一结果引用。
- **FR-012**: 系统 MUST 支持用户指定 Dry Run 模式（`--dry-run` 标志或命令前缀），只生成 ExecutionPlan 与标准化结果（status=simulated），不真正调用子应用；Dry Run 结果 MUST 可与真实执行结果结构对齐（可用于测试与预览）。
- **FR-013**: 系统 MUST 支持通过显式动作语法（如 `hosts:switch(dev)`）旁路自然语言解析，直接调用。
- **FR-014**: 系统 MUST 将 MCP stdio 交互模式下的动作调用保持为可纯文本往返（结构化 JSON 输出）。
- **FR-015**: 系统 MUST 在部分成功时明确列出失败动作及建议（如“以管理员权限重试”）。
- **FR-016**: 系统 SHOULD 缓存近期解析以加速重复命令处理，但 MUST 在执行前重新校验子应用可用性。
- **FR-017**: 系统 MUST 支持对描述文件损坏的子应用进行隔离，不影响其他动作执行。
- **FR-018**: 系统 MUST 对复合命令中互斥的动作检测并提示（例如同一 hosts 分组切换到两个不同分组）。
- **FR-019**: 系统 SHOULD 允许用户通过后缀 `?` 请求解释解析逻辑（可用于调试与信任建立），开启时返回 explain 结构（tokens + matchedRules），默认关闭以最小化开销。
- **FR-020**: 系统 MUST 输出标准化结果对象：`{ overallStatus, actions: [{id, status, reason?, retryHint?}] }`。

### Key Entities _(include if feature involves data)_

- **ApplicationDescriptor**: 子应用标识与动作元数据（id, name, actions[], scheme?, pathValidityState）。
- **ActionDefinition**: 单个可调用动作的语义描述（name, parameters[], category?, requiresElevation?）。
- **ParsedIntent**: 从用户输入获得的结构化动作意图（actionName, targetAppId?, params, confidence, sourceTextSpan）。
- **ExecutionPlan**: 本次输入解析后的执行序列（intents[], deduplicated, conflicts[], strategy=sequential|parallel|mixed）。
- **ActionResult**: 单个动作调用的结果（intentRef, status=success|failed|skipped|timeout, reason?, retryHint?）。
- **CommandHistoryRecord**: 复合命令历史（hash/signature, intentsSummary, timestamp, overallStatus, cacheableFlag）。

---

## Review & Acceptance Checklist

_GATE: Automated checks run during main() execution_

### Content Quality

- [ ] No implementation details (languages, frameworks, APIs)
- [ ] Focused on user value and business needs
- [ ] Written for non-technical stakeholders
- [ ] All mandatory sections completed

### Requirement Completeness

- [ ] No [NEEDS CLARIFICATION] markers remain
- [ ] Requirements are testable and unambiguous
- [ ] Success criteria are measurable
- [ ] Scope is clearly bounded
- [ ] Dependencies and assumptions identified

---

## Execution Status

_Updated by main() during processing_

- [ ] User description parsed
- [ ] Key concepts extracted
- [ ] Ambiguities marked
- [ ] User scenarios defined
- [ ] Requirements generated
- [ ] Entities identified
- [ ] Review checklist passed

---
