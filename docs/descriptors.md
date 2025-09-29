# 应用 Descriptor 规范 (tlfsuite.json)

> 版本：1.0.0 （指“描述文件结构版本”而非应用自身版本）
> 目标：声明一个可被 Launcher / 其它应用发现与调用的本地小工具能力集合。

## 设计原则
- 本地优先：不依赖网络即可解析与调用。
- 最小可用：必填字段极少，允许 actions 为空。
- 可校验：结构化类型 + 语义化版本，便于向后兼容演进。
- 可扩展：预留扩展字段，不破坏现有调用。

## 基本结构
```jsonc
{
  "id": "hosts",                // 唯一应用 ID（小写蛇形 / 短横线，建议 <=32 字符）
  "name": "Hosts Manager",      // 展示名称
  "description": "Manage /etc/hosts rules", // 用户可读描述
  "version": "1.0.0",           // 描述文件结构版本 (SemVer)
  "scheme": "hostsmanager",      // (可选) 自定义 deep link scheme (<scheme>://open?args=...)
  "icon": "icon.png",            // (可选) 图标：相对路径 / data:URL / Linux 图标名
  "actions": [
    {
      "name": "switch_group",    // 动作调用名（唯一，kebab_case 或 snake_case）
      "title": "Switch Group",    // (可选) UI 友好标题
      "args": [
        { "name": "groupId", "type": "string", "required": true }
      ]
    }
  ]
}
```

## 字段说明
| 字段 | 必填 | 类型 | 说明 | 约束 |
|------|------|------|------|------|
| id | 是 | string | 应用唯一标识 | 不含空格；`[a-z0-9-_]+` |
| name | 是 | string | 展示名称 | ≤ 48 chars |
| description | 是 | string | 简要说明 | ≤ 140 chars |
| version | 是 | string | 描述文件结构语义版本 | `MAJOR.MINOR.PATCH` |
| scheme | 否 | string | 自定义 deep link scheme | `[a-z][a-z0-9+.-]*` |
| icon | 否 | string | 图标引用 | data URL 或相对路径 |
| actions | 是 | array | 动作列表（可为空数组） | - |
| actions[].name | 是 | string | 动作调用标识 | `[a-z0-9-_]+` |
| actions[].title | 否 | string | UI 展示用标题 | ≤ 48 chars |
| actions[].args | 否 | array | 参数声明 | - |
| actions[].args[].name | 是 | string | 参数名 | `[a-zA-Z0-9_]+` |
| actions[].args[].type | 是 | string | 参数类型 | `string|number|boolean` |
| actions[].args[].required | 否 | boolean | 是否必填 | 默认 false |

## 版本策略 (version)
- 主版本 (MAJOR) 变化：可能出现不兼容字段调整 → 旧 Launcher 可忽略该应用。
- 次版本 (MINOR) 变化：向后兼容新增（字段可选或新 action）。
- 补丁 (PATCH) 变化：修正文档/描述，无结构变动。
- Launcher 解析策略：
  1. 读取 `version` → 解析 SemVer。
  2. 若 `major` 不在支持集合（例如仅支持 1）→ 忽略并产生日志 `UNSUPPORTED_DESCRIPTOR_VERSION`。
  3. 否则继续校验结构。

## 错误分类与代码
| 场景 | 代码 | 描述 | 行为 |
|------|------|------|------|
| JSON 解析失败 | PARSE_ERROR | 不是合法 JSON | 跳过该应用 |
| 缺失必填字段 | MISSING_FIELD | id/name/description/version/actions 缺失 | 跳过 |
| 字段类型错误 | TYPE_ERROR | 类型与预期不符 | 跳过 |
| 版本不兼容 | UNSUPPORTED_DESCRIPTOR_VERSION | major 不受支持 | 跳过 |
| ID 冲突 | DUPLICATE_APP_ID | 已存在同名 id | 忽略后者 |
| Action 名冲突（同 app） | DUPLICATE_ACTION_NAME | 动作重复 | 后者忽略（记录） |
| 参数声明错误 | ARG_SCHEMA_ERROR | param 缺少 name/type | 忽略该 action |

## 推荐日志字段
```jsonc
{ "event":"DESCRIPTOR_ERROR", "code":"PARSE_ERROR", "appPath":"/path/app", "detail":"..." }
{ "event":"DESCRIPTOR_REGISTERED", "appId":"hosts", "actionCount": 3 }
```

## Deep Link 约定
- 优先自定义：`<scheme>://open?args=<urlencoded>` → args 可为 JSON 字符串 URL 编码。
- 统一入口：`tlfsuite://open?app=<id>&args=<urlencoded>`。
- 安全：解析后仅允许调用 descriptor 中声明的 action。

### args 编码建议
- 形式：将 `{"action":"switch_group","params":{"groupId":"dev"}}` 进行 URL 编码。
- 解析失败 → 返回 VALIDATION_ERROR。

## IPC 调用抽象 (概念)
- 模型：请求/响应，单机进程间通信。
- 约束：
  - 不触发窗口聚焦。
  - 传输为 UTF-8 文本 JSON。
  - 超时（默认 5s）→ 返回 CHANNEL_TIMEOUT。

## 调用统一结果 (InvocationResult)
```jsonc
{
  "status": "success",          // success | error | processing
  "payload": { /* 可选 */ },
  "error": { "code": "...", "message": "..." },
  "meta": { "durationMs": 12 }
}
```

## 示例：最小描述文件
```json
{
  "id": "clipboard",
  "name": "Clipboard Manager",
  "description": "Clipboard history & search",
  "version": "1.0.0",
  "actions": []
}
```

## 示例：含自定义 scheme 与 action
```json
{
  "id": "hosts",
  "name": "Hosts Manager",
  "description": "Manage /etc/hosts rules",
  "version": "1.0.0",
  "scheme": "hostsmanager",
  "actions": [
    {
      "name": "switch_group",
      "title": "Switch Group",
      "args": [ { "name": "groupId", "type": "string", "required": true } ]
    },
    {
      "name": "list_groups",
      "title": "List Groups",
      "args": []
    }
  ]
}
```

## 未来扩展 (不影响 1.x 兼容)
| 字段 | 目标 | 示例 |
|------|------|------|
| categories[] | 分类检索 | ["network","system"] |
| permissions[] | 显式敏感能力声明 | ["fs.read","network.dns"] |
| engines | 版本兼容范围 | { "launcher": ">=1.0 <2.0" } |
| tags[] | 语义搜索 | ["hosts","dns","switch"] |

## 编写清单 (Checklist)
- [ ] 必填字段全部存在
- [ ] 语义化 version 合法
- [ ] actions 名称唯一
- [ ] 参数声明完整 (name + type)
- [ ] 不含未知主版本

---
如需自动校验：后续可用 `descriptor:validate` 命令（规划中）。
