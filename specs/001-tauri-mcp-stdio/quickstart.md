# Quickstart: 智能底座意图解析 & 执行

目标：演示从输入到执行/历史的完整最小路径 + Dry Run。

## 前置

- 已有 `apps/launcher` 运行 (pnpm -C apps/launcher tauri dev)
- 已至少 2 个子应用含 `tlfsuite.json` (示例: hostsManager, clipboard)

## 步骤

1. 启动主应用。
2. 打开命令面板或输入框，键入：
   ```
   启用开发环境 hosts 规则并打开剪贴板历史
   ```
3. 系统应：
   - 解析出 intents ≥2 (hosts:switch(dev), clipboard:openHistory)
   - 去重 / 生成执行计划 (parallel or mixed)
   - 执行并显示聚合结果。
4. 观察结果面板：成功动作、耗时、部分失败（如权限）提示。
5. 30 秒内再次输入相同指令：应命中历史缓存加速（解析时间显示更低）。
6. 触发 Dry Run：
   ```
   --dry-run 启用开发环境 hosts 规则并打开剪贴板历史
   ```
   - 应仅展示 status=simulated，包含 predictedEffects。
7. Explain 模式 (若启用开关/配置)：
   ```
   启用开发环境 hosts 规则并打开剪贴板历史 ?
   ```
   - 返回 explain 区域（tokens + matchedRules）。
8. 查看历史：调用 list_history (UI 或调试命令) → 最近记录存在且未超过 30 天窗口。

## 验证

| 检查项   | 预期                                    |
| -------- | --------------------------------------- |
| 并发上限 | 超过 4 动作时排队                       |
| 超时     | 人为使某动作阻塞 >5s → status=timeout   |
| 去重     | 输入中重复相同动作只执行一次            |
| 冲突     | 构造冲突动作对 → 返回冲突提示或强制顺序 |
| Dry Run  | 不触发任何副作用 (验证外部状态未变)     |
| 历史清理 | 人工插入旧记录 (>30 天) 后调用清理触发  |

## 故障排查

- 未解析出意图：检查关键词映射表是否包含“hosts”/“剪贴板”。
- 超时过多：确认 semaphore 并发未被错误设为 0；查看是否死锁阻塞 UI 线程。
- Explain 空：确保显式 `?` 或配置已开启。

-- END --
