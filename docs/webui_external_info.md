# WebUI 外部信息获取方式梳理

本文梳理 `webui` 当前所有「从应用外部读取信息」的入口、来源、调用链与降级策略，便于后续维护与扩展。

## 1. 总览

`webui` 的外部信息主要来自 4 类来源：

1. **KernelSU 执行结果（设备本地）**
   - 通过动态导入 `kernelsu` 后调用 `exec(cmd)` 获取。
   - 覆盖：配置文件、模块列表、磁盘占用、系统信息、设备型号/安卓版本、程序版本。

2. **GitHub HTTP API（网络）**
   - `InfoTab` 使用浏览器 `fetch` 拉取仓库贡献者及其详情。

3. **浏览器本地存储（localStorage）**
   - 用户偏好：语言、底部导航修复开关。
   - 信息缓存：贡献者列表（1 小时缓存）。

4. **Mock 数据（开发/降级）**
   - 当 `import.meta.env.DEV` 或 `kernelsu` 不可用时，统一走 `MockAPI`。

---

## 2. 外部信息入口与判定逻辑

### 2.1 API 实现选择（Real / Mock）

文件：`webui/src/lib/api.ts`

- 启动时尝试动态导入 `kernelsu`：
  - 成功：拿到 `ksu.exec`，可走真实设备命令。
  - 失败：记录 warning，进入 mock 模式。
- 判定条件：
  - `shouldUseMock = import.meta.env.DEV || !ksuExec`
- 导出统一入口：
  - `export const API = shouldUseMock ? MockAPI : RealAPI`

这保证了上层 `store` 和页面组件不需要区分真实/模拟来源。

---

## 3. KernelSU（RealAPI）信息获取清单

文件：`webui/src/lib/api.ts`

### 3.1 配置读取：`loadConfig()`

- 来源：`/data/adb/magic_mount/config.toml`（常量来自 `PATHS.CONFIG`）
- 方式：执行 shell
  - `[ -f "${PATHS.CONFIG}" ] && cat "${PATHS.CONFIG}" || echo ""`
- 解析：`parseKvConfig(stdout)`
- 失败回退：`DEFAULT_CONFIG`

### 3.2 配置保存：`saveConfig(config)`

- 写入路径同上。
- 方式：`mkdir -p` + heredoc `cat > ...` + `chmod 644`
- 错误处理：`errno !== 0` 抛出异常。

> 注：保存本身不是“读取外部信息”，但会影响后续 `loadConfig` 的外部读取结果。

### 3.3 模块扫描：`scanModules()`

- 来源：`meta-mm` 命令输出 JSON。
- 方式：
  - `/data/adb/modules/magic_mount_rs/meta-mm scan --json`
- 处理：`JSON.parse(stdout)` 并映射为 `MagicModule[]`
- 失败回退：`[]`

### 3.4 存储占用：`getStorageUsage()`

- 来源：`df` 命令。
- 方式：`df -k /data/adb/modules | tail -n 1`
- 提取：总量/已用/百分比，格式化为可读字符串。
- 失败回退：`{ size:"-", used:"-", percent:"0%", type:null }`

### 3.5 系统信息：`getSystemInfo()`

- 来源：`uname` / `getenforce` / `ls`。
- 方式：
  - `uname -r` -> 内核版本
  - `getenforce` -> SELinux 状态
  - `ls -1 /data/adb/modules` -> 活跃挂载模块目录
- 处理：过滤 `magic_mount_rs` 本体目录。
- 失败回退：`kernel/selinux/mountBase` 置 `-`，`activeMounts` 置空数组。

### 3.6 设备基础信息：`getDeviceStatus()`

- 来源：`getprop`。
- 方式：
  - `getprop ro.product.model`
  - `getprop ro.build.version.release`
- 返回：`model` / `android`，其余字段由 `store` 用 `getSystemInfo` 结果覆盖。

### 3.7 版本信息：`getVersion()`

- 来源：`meta-mm version` JSON。
- 方式：`/data/adb/modules/magic_mount_rs/meta-mm version`
- 返回：`res.version ?? "0.0.0"`
- 失败回退：`"Unknown"`

### 3.8 外部跳转与设备动作（附）

- `openLink(url)`：执行 `am start -a android.intent.action.VIEW -d "..."`
- `reboot()`：执行 `svc power reboot || reboot`

这两项不“获取信息”，但同属外部系统交互入口。

---

## 4. GitHub 网络信息获取

文件：`webui/src/routes/InfoTab.tsx`

### 4.1 贡献者列表

- URL：
  - `https://api.github.com/repos/originalFactor/meta-magic_mount-rs/contributors`
- 流程：
  1. 拉取基础贡献者列表。
  2. 过滤 bot（`type === "Bot"` 或 `login` 包含 `bot`）。
  3. 对每个贡献者继续请求 `user.url` 获取 `name` / `bio`。
  4. `Promise.all` 汇总后渲染。

### 4.2 缓存策略

- 本地缓存键：`mm_contributors_cache`
- 有效期：`1000 * 60 * 60`（1 小时）
- 命中缓存：直接使用缓存并跳过网络。
- 缓存损坏：删除缓存并重新请求。

### 4.3 异常处理

- 任意网络异常 -> `setError(true)`，UI 显示 `loadFail` 文案。

---

## 5. localStorage 的外部信息读取

### 5.1 用户偏好读取（store 初始化）

文件：`webui/src/lib/store.ts`

- `mm-lang`：读取语言，默认 `en`。
- `mm-fix-nav`：读取底部导航修复开关，值为字符串 `"true"` 时启用。

### 5.2 用户偏好写入

- `setLang(code)` 写入 `mm-lang`
- `toggleBottomNavFix()` 写入 `mm-fix-nav`

### 5.3 业务缓存读写

文件：`webui/src/routes/InfoTab.tsx`

- `fetchContributors()` 读取/写入 `mm_contributors_cache`。

---

## 6. 页面层调用链（数据流）

### 6.1 应用启动链路

1. `App` 在 `onMount` 调用 `store.init()`。
2. `store.init()` 并行触发：
   - `loadConfig()` -> `API.loadConfig()`
   - `loadStatus()` -> 多个外部查询

### 6.2 状态页链路（StatusTab）

- `StatusTab` 在 `onMount` 触发 `store.loadStatus()`。
- `store.loadStatus()` 外部调用顺序：
  1. `API.getDeviceStatus()`
  2. `API.getVersion()`
  3. `API.getStorageUsage()`
  4. `API.getSystemInfo()`
  5. 若模块为空，补 `loadModules()` -> `API.scanModules()`
- 合并逻辑：用 `sysInfo.kernel/selinux` 覆盖 `baseDevice` 对应字段。

### 6.3 模块页链路（ModulesTab）

- `onMount` 调用 `store.loadModules()` -> `API.scanModules()`。

### 6.4 配置页链路（ConfigTab）

- 保存：`store.saveConfig()` -> `API.saveConfig()`
- 重载：`store.loadConfig()` -> `API.loadConfig()`

### 6.5 信息页链路（InfoTab）

- `onMount`：
  - `API.getVersion()`（显示版本）
  - `fetchContributors()`（缓存优先，网络兜底）
- 外链点击：`API.openLink(url)`

---

## 7. Mock 模式数据来源

文件：`webui/src/lib/api.mock.ts`

- 所有接口返回固定或延时模拟数据（`MOCK_DELAY = 600ms`）。
- `openLink` 直接 `window.open`。
- `reboot` 为弹窗提示，不执行真实重启。

用途：本地开发、无 KernelSU 环境下的界面联调与演示。

---

## 8. 结论（维护建议）

1. **外部信息采集职责集中在 `API` 层**，页面不直接执行系统命令，结构清晰。
2. **网络信息仅在 InfoTab 使用 GitHub API**，并有短时缓存，已具备基础限流能力。
3. **设备信息在 `store.loadStatus()` 统一聚合**，是状态页/顶部展示的关键入口。
4. 后续若扩展新的外部信息源，建议优先新增到 `APIType` 并由 `store` 统一编排，以保持当前分层一致性。
