# MVP Issue Breakdown: Multi-App Descriptor & Invocation

> 对应规范：`specs/001-tauri-rust-mcp/spec.md`
> MVP 范围：FR-001~008, FR-010~013, FR-016, FR-018, FR-023, FR-024

## 说明
- 优先级：P0 (必须) / P1 (重要) / P2 (可延后)
- 标签建议：`mvp` `invocation` `descriptor` `ipc` `cli` `docs`
- 完成 Definition of Done (DoD)：
  1. 通过基本测试/用例验证
  2. 没有未处理 panic 或未捕获错误
  3. 符合日志规范（无敏感信息）
  4. 文档或注释最少描述 public 接口语义

---

## Epic 结构
| Epic | 子任务 | 关联 FR |
|------|--------|---------|
| Descriptor 发现与注册 | 扫描 / 解析 / 版本兼容 / Registry / 热刷新 | 001-004,010,023,024 |
| Invocation 通路 | 参数校验 / Deep Link / IPC 抽象 / 执行管线 / 结果格式 | 005-008,018 |
| CLI 入口 | 命令解析 / 调用输出 | 013,018 |
| 错误与冲突处理 | NOT_FOUND / 版本不兼容 / 冲突策略 | 003,016,024 |
| 文档与测试 | 用例、开发指引 | 场景 & 全部 |

---

## 任务列表（建议转换为单个 Issue + 子任务或多个 Issue）

### 1. Freeze MVP Scope (P0)
- 目标：确认范围锁定，记录延后 FR 列表
- 输出：更新 `BACKLOG.md`（如不存在则新建）
- 验收：文件列出 FR-009/014/015/017/019/020/021/022/025

### 2. Descriptor Schema 定义 (P0)
- 内容：字段、必填、类型、示例、错误码（结构错误 vs 版本不兼容）
- 输出：`docs/descriptors.md` + JSON Schema (`docs/descriptors.schema.json` 占位)
- 验收：可用示例通过 schema 校验

### 3. 扫描器实现 (P0)
- 路径顺序：exe 同级 → bundle resources → share 路径
- 忽略策略：文件不存在静默；JSON 解析错误 -> 警告日志 + 跳过
- 验收：模拟 2 个合法 + 1 个损坏 descriptor 结果正确

### 4. 版本兼容校验 (P0)
- 规则：仅支持主版本 = 1（示例）
- 不兼容：打日志 code=UNSUPPORTED_DESCRIPTOR_VERSION
- 验收：主版本 2 被跳过，不影响其他

### 5. Registry 结构 (P0)
- 结构：`AppRegistry { apps: Map<id,App>, actions: Map<id,ActionMeta[]> }`
- 冲突：重复 appId -> 记录 `DUPLICATE_APP_ID` 忽略后者
- 验收：重复 id 测试通过

### 6. 热刷新机制 (P1)
- 监听：轮询或文件 watcher（抽象接口便于替换）
- 去抖：100~200ms
- 验收：修改 descriptor 动态刷新 actions

### 7. 参数校验模块 (P0)
- 输入：Action 定义 + 用户参数（Map）
- 输出：校验结果（ok | error{code,fields[]}）
- 错误：MISSING_ARG / TYPE_ERROR
- 验收：缺参/类型不符用例通过

### 8. IPC 通道抽象 (P1)
- 接口：`trait Channel { fn invoke(req)->Result<InvocationResult,ChannelError> }`
- MVP 实现：本地 loopback（假实现：调用直接回调）
- 错误：TIMEOUT / CHANNEL_UNAVAILABLE
- 验收：模拟超时路径

### 9. Deep Link 适配 (P0)
- 解析：app id + args（URL 解码 JSON / k=v 列表二选一）
- 安全：拒绝未知 action
- 验收：示例 link 触发 action

### 10. Invocation 管线 (P0)
- 步骤：解析 → 校验 → 路由 → 执行 → 结构化结果 → 计时
- 错误分类：NOT_FOUND / VALIDATION_ERROR / EXECUTION_ERROR
- 验收：计时字段 meta.durationMs >0

### 11. 结果结构固定 (P0)
- 格式：`{status:'success'|'error', payload?, error?{code,message}, meta{durationMs}}`
- 验收：所有调用返回该结构且字段覆盖率 ≥ 90%（测试统计）

### 12. CLI 入口 (P0)
- 形式：`launcher --action app:action --args key=value --json`
- 输出：JSON（无彩色，无多余日志）
- 错误码：程序 exit code !=0
- 验收：脚本调用可管道解析

### 13. NOT_FOUND / 冲突处理 (P0)
- 行为：未知 app/action -> NOT_FOUND；冲突 appId 早已在 Registry 处理
- 验收：测试脚本稳定复现

### 14. 日志事件集 (P0)
- 列表：SCAN_START / SCAN_END / DESCRIPTOR_ERROR / INVOCATION_{SUCCESS|ERROR}
- 格式：结构化（JSON 行 / key=value）
- 验收：日志采集解析成功

### 15. 验收测试编写 (P0)
- 覆盖：Acceptance Scenarios 1~7 + 错误用例
- 方式：脚本 or 单元测试
- 验收：全部绿色

### 16. 开发者指南 (P1)
- 内容：如何新增应用 / 字段说明 / 调用示例 / 版本升级策略
- 输出：`docs/developer-guide.md`
- 验收：新成员只读文档能完成示例新增

### 17. Backlog 列表 (P2)
- 内容：非 MVP FR 映射 → 潜在设计走向
- 输出：`BACKLOG.md`
- 验收：含每项一句话价值说明

---

## 建议 Issue 模板（复制使用）
```
### 背景
(简述本任务目的)

### 需求对应 FR
- FR-XXX

### 验收标准
- [ ] ...
- [ ] ...

### 日志/错误码
- 相关：...

### 依赖
- #<issue>

### 备注
(风险/权衡)
```

## 风险与缓解
| 风险 | 影响 | 缓解 |
|------|------|------|
| 过早定 IPC 实现 | 返工 | 先抽象 trait + stub |
| 过多同步文件 IO | 启动慢 | 缓存 + 延迟扫描 |
| Deep Link 参数不统一 | 用户体验差 | 制定统一 args 编码格式 (优先 JSON URL 编码) |
| 规范漂移 | 测试失效 | MVP 范围冻结 + 评审流程 |

## 后续指标（可选）
- 发现耗时 P50 < 120ms, P95 < 200ms (10 apps)
- 跨应用调用成功率 > 99%（本地测试）
- 规范升级主版本频率 ≤ 每季度 1 次

---

如需：我可以继续生成 `descriptors.md` 与初始 JSON Schema 骨架。告知即可。
