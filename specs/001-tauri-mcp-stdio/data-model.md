# Phase 1 Data Model

**Feature**: 本地优先多应用智能底座（MCP stdio 交互）  
**Date**: 2025-09-30  
**Source**: spec.md + research.md

## 1. Entity Catalog

| Entity | Purpose | Persistence | Notes |
| --- | --- | --- | --- |
| ApplicationDescriptor | 描述子应用及其可执行动作 | Derived (scan) | 不持久化，扫描缓存可内存 + 失效策略 |
| ActionDefinition | 子应用具体动作元数据 | Derived | 嵌套于 ApplicationDescriptor |
| ParsedIntent | 用户输入解析出的动作意图 | Ephemeral | Dry Run / 执行期内存 |
| ExecutionPlan | 解析+去重+冲突检测后执行计划 | Ephemeral | 含批次/并行控制信息 |
| ExecutionPlanBatch | 并行批次容器 | Ephemeral | 每批并行 <= max concurrency |
| ActionResult | 单动作执行输出 | Persist (partial) | 持久化摘要到历史 |
| CommandHistoryRecord | 历史复合命令记录 | Persist | 滚动 30 天删除 |
| DescriptorLoadIssue | 描述文件加载问题 | Ephemeral (optional log) | 可聚合显示 |
| ConflictDetection | 冲突检测结果 | Ephemeral | plan 生成时产生 |

## 2. Detailed Schemas (TypeScript-ish + Rust Parity)

### ApplicationDescriptor

```
interface ApplicationDescriptor {
  id: string;            // stable unique id
  name: string;
  scheme?: string;       // optional deeplink scheme
  actions: ActionDefinition[];
  path: string;          // filesystem root
  pathValidity: 'ok' | 'missing' | 'inaccessible';
  issues?: DescriptorLoadIssue[]; // optional collected issues
}
```

### ActionDefinition

```
interface ActionDefinition {
  name: string;                       // e.g. switch, openHistory
  parameters?: ParameterSpec[];       // simple name:type
  category?: string;                  // grouping (hosts, clipboard)
  requiresElevation?: boolean;
  conflictKey?: string;               // 用于互斥检测
  description?: string;               // For explain mode
}

interface ParameterSpec {
  name: string;
  type: 'string' | 'number' | 'boolean' | 'enum';
  required: boolean;
  enumValues?: string[];
}
```

### ParsedIntent

```
interface ParsedIntent {
  id: string;                    // uuid
  actionName: string;
  targetAppId?: string;          // resolved app id if determinable
  params: Record<string, unknown>;
  confidence: number;            // 0..1
  sourceTextSpan: { start: number; end: number; };
  explicit: boolean;             // 来自显式语法 (app:action())
}
```

### ExecutionPlan & Batch

```
interface ExecutionPlanBatch {
  batchId: string;
  intents: ParsedIntent[];               // 同批并行
}

interface ExecutionPlan {
  planId: string;
  originalInput: string;
  intents: ParsedIntent[];               // 全量（含去重前? -> 使用 deduplicated 字段）
  deduplicated: ParsedIntent[];          // 去重后
  batches: ExecutionPlanBatch[];         // 并行批次
  conflicts: ConflictDetection[];        // 互斥列表
  strategy: 'sequential' | 'parallel' | 'mixed';
  generatedAt: number;
  dryRun: boolean;
  explain?: ExplainPayload;              // explain 模式
}
```

### ConflictDetection

```
interface ConflictDetection {
  conflictKey: string;
  intents: string[];              // intent ids
  reason: string;                 // e.g. 'mutually-exclusive-hosts-group'
  resolution: 'force-order' | 'user-select' | 'drop-conflicting';
}
```

### ExplainPayload

```
interface ExplainPayload {
  tokens: string[];
  matchedRules: { ruleId: string; weight: number; intentId?: string }[];
}
```

### ActionResult

```
interface ActionResult {
  intentId: string;
  status: 'success' | 'failed' | 'skipped' | 'timeout' | 'simulated';
  reason?: string;                 // failure / skip explanation
  retryHint?: string;              // e.g. 're-run with elevation'
  predictedEffects?: string[];     // Dry Run 补充
  durationMs?: number;             // 实际耗时 (simulated 可省略 or 0)
  startedAt: number;
  finishedAt?: number;
}
```

### CommandHistoryRecord

```
interface CommandHistoryRecord {
  signature: string;                 // blake3 hash of normalized intents
  input: string;                     // 原始输入
  intentsSummary: string[];          // e.g. ['hosts:switch(dev)', 'clipboard:openHistory']
  overallStatus: 'success' | 'partial' | 'failed';
  createdAt: number;
  planSize: number;                  // intents count
  explainUsed: boolean;
}
```

### DescriptorLoadIssue

```
interface DescriptorLoadIssue {
  appId?: string;          // may be unknown if parse failed early
  level: 'PARSE_ERROR' | 'SCHEMA_ERROR' | 'SEMANTIC_ERROR';
  message: string;
  path: string;            // file path
}
```

## 3. Relationships

- ApplicationDescriptor 1..\* ActionDefinition
- ExecutionPlan 1..\* ParsedIntent
- ExecutionPlan 1.._ ExecutionPlanBatch 1.._ ParsedIntent
- ExecutionPlan 1..\* ConflictDetection
- CommandHistoryRecord summarizes one ExecutionPlan (not stored directly to avoid bloat)
- ActionResult links to ParsedIntent via intentId

## 4. Validation Rules

| Entity                | Field      | Rule                                                 |
| --------------------- | ---------- | ---------------------------------------------------- |
| ApplicationDescriptor | id         | non-empty, unique (scan set)                         |
| ActionDefinition      | name       | non-empty, unique per descriptor                     |
| ParsedIntent          | confidence | 0 <= value <= 1                                      |
| ExecutionPlan         | batches    | 每批 intents.length <= concurrency 上限              |
| ExecutionPlan         | conflicts  | 若存在同 conflictKey 多意图 → 必须出现在 conflicts   |
| ActionResult          | status     | 若 timeout 则 finishedAt 可为生成时；reason 推荐填充 |
| CommandHistoryRecord  | signature  | unique within retention window                       |

## 5. State Transitions (Intent Lifecycle)

ParsedIntent(created) → (deduplicated?) → (conflict evaluated) → scheduled(batch) → executing → result(ActionResult) → aggregated(overallStatus) → history persisted

## 6. Derived / Computed Fields

- CommandHistoryRecord.signature: normalize(intents) → JSON → blake3
- ExecutionPlan.strategy: if conflicts>0 or sequential-only actions detected → 'mixed' else if batches.len>1 → 'parallel' else 'sequential'
- overallStatus: reduce(ActionResult.status)

## 7. Open Considerations

- History storage medium: SQLite table `command_history(signature TEXT PK, input TEXT, status TEXT, created_at INT, plan_size INT, explain_used INT)`.
- Potential index: (created_at) for purge.

## 8. Rust Type Parity Notes

- Use `serde` derive for all public structs for IPC serialization.
- Prefer `SmallVec` for small fixed-size collections (batches) to reduce alloc.

## 9. Future Extension Hooks

- SemanticMatcher trait for embedding-based retrieval.
- Telemetry adapter (kept disabled by default) for local metrics.

-- END --
