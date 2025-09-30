# Phase 0 Research: 本地优先多应用智能底座（MCP stdio 交互）

**Scope**: 解析 → 计划 → 调度 → 结果聚合 → 历史缓存。仅研究不写实现代码。 **Inputs**: spec.md, plan.md Technical Context 未决条目。 **Date**: 2025-09-30

## 1. 未决与问题清单

| ID | 问题 | 描述 | 影响 | 优先级 | 解决状态 |
| --- | --- | --- | --- | --- | --- |
| U1 | FR-019 解释模式 | 是否提供用户可见的解析 explain | 功能可见性/调试 | M | OPEN |
| U2 | 解析策略选择 | 规则/关键词 vs 轻量 embedding (本地) | 体积 & 准确度 | H | OPEN |
| U3 | 历史签名生成 | 如何规避参数顺序差异 | 缓存命中 | M | OPEN |
| U4 | 并发执行集成 | Rust tokio runtime 与 Tauri 主线程交互 | 稳定性 | H | OPEN |
| U5 | Dry Run 结果结构 | 是否需要副作用预测字段 | 一致性 | M | OPEN |
| U6 | 冲突检测规则 | 如何定义互斥动作类别 | 正确性 | M | OPEN |
| U7 | 描述文件损坏分类 | 分层错误模型（解析/结构/语义） | 错误隔离 | M | OPEN |

## 2. 研究与决策

### U2: 解析策略选择

- 备选方案:
  1. 纯规则 + 关键词映射 (Trie / HashMap) + 模糊阈值 (Levenshtein)
  2. 轻量 embedding (e.g. minisearch / 本地 vector) + 语义相似度 (Cosine)
  3. 外部大型模型（排除：违背最小依赖 + 离线体积）
- 评价:
  - 方案 1 体积极小, 可预测, 维护成本低; 语义泛化弱。
  - 方案 2 需要引入额外索引 & vector 库, 增加初始加载与内存, 但提升模糊匹配。
- 决策: Phase 1 采用 方案 1。保留向方案 2 升级钩子（接口留: `SemanticMatcher` trait, 初始实现为空）。
- Rationale: 满足当前 FR 覆盖; 避免 premature complexity。
- Alternatives: 方案 2 将在子应用动作>50 且模糊失败率 >15% 时重新评估。

### U4: 并发执行集成

- 选项: (a) 每次命令生成 Tokio runtime (开销大), (b) 复用全局 runtime (Tauri async 兼容), (c) rayon 线程池。
- 决策: 复用 Tauri 自带/单一 tokio runtime (若已存在) + 限制并发 semaphore (max 4)。
- Rationale: 减少线程爆炸; 更好 timeout 控制。
- Alternatives: rayon 不易统一 timeout/async I/O; 临时 runtime 造成重复初始化延迟。

### U3: 历史签名生成

- 需求: 语义等价 (参数顺序不同) 仍命中。
- 签名算法: 正规化 → (动作名排序 + 目标应用排序 + 归一参数键值排序) → JSON 串 → blake3 hash (快速, 稳定)。
- 决策: 使用 blake3 (Rust crate 体积小)。
- Rationale: 高速、低碰撞、无需加密特性。
- Alternatives: SHA256 (较慢, 过度), murmur (无现成稳定 crate?).

### U5: Dry Run 结果结构

- 需求: 与真实结构一致, 但不执行副作用。
- 决策: `status=simulated` + 增补 `predictedEffects?: string[]` (可选, 列出静态描述: “修改 hosts”, “打开剪贴板窗口”)。
- Rationale: 便于前端显示“将要发生”列表, 不破坏结构化消费。
- Alternatives: 单独类型 (导致分支) → 拒绝。

### U6: 冲突检测规则

- 示例: 同一 hosts 分组切换到不同分组; 相同资源互斥写操作。
- 决策: 为 ActionDefinition 可选字段 `conflictKey`。计划构建时若同批内出现同 conflictKey 且语义不一致 → 标记互斥并要求拆分或按顺序执行。
- Rationale: 通用扩展点, 不局限 hosts。
- Alternatives: 硬编码 per-app 逻辑, 可扩展性差。

### U7: 描述文件损坏分类

- 分级:
  1. PARSE_ERROR: JSON 语法错误
  2. SCHEMA_ERROR: 缺核心字段 (id/actions)
  3. SEMANTIC_ERROR: 字段冲突或无动作
- 决策: 扫描阶段输出 `DescriptorLoadIssue { appPath, level, message }` 并隔离该应用（不列入动作集合）。
- Rationale: 精确诊断, UI 分级显示。
- Alternatives: 单一错误字符串（缺少可视化价值）。

### U1: FR-019 解释模式

- 价值: 提升信任与调试; 能显示解析步骤。
- 成本: 增加日志/结构; 可能暴露内部匹配策略；UI 额外分支。
- 决策: 纳入 (升级 SHOULD→MUST?) 保持为可配置：默认关闭; 开启时返回 `explain?: { tokens: string[]; matchedRules: {ruleId, weight}[] }`。
- Rationale: 对后期引入语义匹配有价值; 低体积成本 (规则模式不大)。
- Resolution: 将 FR-019 从 SHOULD + [NEEDS CLARIFICATION] → SHOULD (解释模式可选开关)；去除 NEEDS CLARIFICATION 标记。

## 3. 决策汇总矩阵

| 决策             | 结果                   | 状态     | 触发再评估条件             |
| ---------------- | ---------------------- | -------- | -------------------------- |
| 解析策略         | 规则 + 预留语义接口    | ACCEPTED | 动作数>50 且模糊失败率>15% |
| 并发模型         | 单 runtime + semaphore | ACCEPTED | 队列等待>1s 平均           |
| 签名算法         | 归一 JSON → blake3     | ACCEPTED | 碰撞>0.1%                  |
| Dry Run 扩展     | predictedEffects       | ACCEPTED | 前端不使用此字段           |
| 冲突检测         | conflictKey 属性       | ACCEPTED | 冲突漏检 >5%               |
| 描述文件错误分类 | 三级错误模型           | ACCEPTED | 复杂度>收益时简化          |
| 解释模式         | 可选 explain 结构      | ACCEPTED | 用户无使用反馈             |

## 4. 更新需求 (Spec Patch 指引)

- FR-019: 移除 [NEEDS CLARIFICATION]，保留 SHOULD；描述新增 explain 可选字段输出。
- 新增：ActionResult.simulated? 不需要（用 status 即可）；Dry Run 补充 predictedEffects 可选数组（文档说明）。
- 新增实体: DescriptorLoadIssue, ConflictDetection。

## 5. 风险与缓解

| 风险         | 描述                 | 等级 | 缓解                                   |
| ------------ | -------------------- | ---- | -------------------------------------- |
| 匹配误判     | 规则匹配过度或不足   | M    | 增加信心阈值 + explain 模式审计        |
| 队列阻塞     | 长动作阻塞其他执行   | M    | 每动作独立超时 5s, 超时释放槽位        |
| 历史膨胀     | 数据库未及时清理     | L    | 启动与写入时触发滚动删除 SQL           |
| 解析代码膨胀 | 语义升级时引入大依赖 | L    | 预留接口, 独立 crate, 延迟加载         |
| UI 复杂性    | 结果聚合视图过多状态 | M    | 统一状态机 overallStatus + action tags |

## 6. 后续设计输入 (供 Phase 1)

- 数据模型需定义：Intent, ExecutionPlanBatch, ActionResult(predictedEffects?), CommandHistoryRecord, DescriptorLoadIssue。
- 契约：Tauri commands 输入输出 JSON schema；`parse_intent`, `dry_run`, `execute_plan`, `list_history`。
- 测试清单：
  1. 去重 & 冲突检测单元测试
  2. 并发上限控制（模拟 >4 动作）
  3. 超时后其他动作仍完成
  4. Dry Run 结构匹配真实执行形状
  5. 历史 30 天滚动清理
  6. explain 模式开关正确输出

## 7. 结论

所有 OPEN 未决均已给出决策；无阻碍进入 Phase 1。Complexity Tracking 暂无条目。

-- END --
