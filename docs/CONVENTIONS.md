# 交互约定（Apps 间）

状态：🔜/🚧/✅/⏸️（不写日期）

## 深链（Deeplink）
- Scheme：`tlfsuite://`
- 格式：`tlfsuite://open?app=<id>&args=<urlencoded>`
- 示例：`tlfsuite://open?app=hosts&args=rule%3Ddev`
- 目标：启动/切换到对应应用或在已打开的窗口内路由到模块。

## 事件（Event Bus）
- 频道：`launcher://open`
- 载荷：
```ts
interface OpenEventPayload {
  app: string;      // 目标 app id，例如 'hosts'
  args?: string;    // 约定为 querystring，或 JSON 字符串
}
```

## 命令（IPC）
- `open_with_args(appName: string, args?: string)`
  - 行为：在 Launcher 内广播 `launcher://open` 事件，供其它窗口/模块响应。
  - 约束：仅允许白名单 appName（后续通过配置强化）。

## 推荐优先级
1. 事件优先：同一进程或同一应用内模块间通讯
2. 深链互通：跨应用/跨进程唤起
3. 命令参数：系统层级启动时的路由分发（例如注册自定义协议处理）
