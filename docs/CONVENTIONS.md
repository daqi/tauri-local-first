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
- `open_with_args(app_name: string, args?: string)`
  - 行为：构造 deep link（优先应用自定义 scheme，否则 `tlfsuite://open?app=<id>`）并通过系统 CLI (`open` / `start` / `xdg-open`) 启动。
  - 约束：app_name 应在已扫描出的 descriptor 集合内。

## 推荐优先级
1. 事件优先：同一进程或同一应用内模块间通讯
2. 深链互通：跨应用/跨进程唤起
3. 命令参数：系统层级启动时的路由分发（例如注册自定义协议处理）

---

## 应用 Descriptor（`tlfsuite.json`）

放置位置（按优先匹配顺序）：

1. `<appRoot>/tlfsuite.json`
2. `<appRoot>/Contents/Resources/tlfsuite.json` (macOS .app)
3. `<appRoot>/resources/tlfsuite.json`
4. `<appRoot>/share/tlfsuite/tlfsuite.json`

示例：
```jsonc
{
  "id": "hosts",
  "name": "Hosts Manager",
  "description": "Manage /etc/hosts rules",
  "scheme": "hostsmanager",
  "actions": [
    { "name": "open" },
    { "name": "rule", "args": [{ "name": "id", "type": "string", "required": true }] }
  ],
  "icon": "icon.png"
}
```

## 扫描根目录来源
- 环境变量：`TLFSUITE_APPS_DIR`（多个路径，系统路径分隔符）
- 平台适配：
  - macOS: `/Applications`
  - Linux: `.local/share/applications`, `/usr/share/applications` 及 `.desktop` 解析出的可执行父目录
  - Windows: 注册表 Uninstall 项的 `InstallLocation` / `DisplayIcon` 父目录

## 图标解析优先级
1. descriptor.icon：
   - `data:` 直接使用
   - Linux 名称：XDG Icon Theme 解析（主题/尺寸/上下文）
   - 路径：相对 descriptor 目录或绝对路径
2. macOS `.app`：Info.plist → `.icns` 使用 `sips` 转 PNG
3. 兜底：`icon.png`, `icons/icon.png`, (macOS) `Contents/Resources/icon.png`，(Windows) 追加 `.ico` 变体

