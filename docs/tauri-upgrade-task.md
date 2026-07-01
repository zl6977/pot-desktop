# Tauri 1.x → 2.x 升级任务书

## 背景

Arch Linux 官方仓库已从 `extra` 中移除 `webkit2gtk`（-4.0 API），只提供 `webkit2gtk-4.1`（预编译，2.52.4）。老版 `webkit2gtk 2.50.6` 退居 AUR，需要本地编译，资源消耗大。

当前项目使用 **Tauri 1.8**，底层 Rust crate 依赖链为：

```
tauri 1.8 → wry 0.24.x → webkit2gtk 0.18.2 → pkg-config webkit2gtk-4.0
```

`pkg-config` 查找 `webkit2gtk-4.0`，但系统上只有 `webkit2gtk-4.1.pc`，编译失败。

## 升级目标

| 组件 | 当前版本 | 目标版本 |
|------|---------|---------|
| Tauri | 1.8 | 2.x |
| Wry | 0.24.x | 0.55.x |
| webkit2gtk (Rust) | 0.18.2 | 2.0.2 |
| webkit2gtk-sys (Rust) | 0.18.0 | 2.0.2 |
| pkg-config 查找名 | `webkit2gtk-4.0` | `webkit2gtk-4.1` |
| soup | libsoup2 | libsoup3 |
| gtk | 0.16.x | 0.18.x |

## 辅助工具

安装 `migrate-deps` 辅助版本号升级：

```bash
cargo install migrate-deps
migrate-deps
```

> **注意**：`migrate-deps` 只改版本号，不改代码。API 破坏性变更需手动处理。

## 需要修改的文件

### 1. `src-tauri/Cargo.toml`

```diff
- tauri = { version = "1.8", features = [...] }
- tauri-build = { version = "1.5", features = [] }
+ tauri = { version = "2.x", features = [...] }
+ tauri-build = { version = "2.x", features = [] }

- tauri-plugin-single-instance = { git = "...", branch = "v1" }
- tauri-plugin-autostart = { git = "...", branch = "v1" }
- tauri-plugin-fs-watch = { git = "...", branch = "v1" }
- tauri-plugin-store = { git = "...", branch = "v1" }
- tauri-plugin-log = { git = "...", branch = "v1" }
- tauri-plugin-sql = { git = "...", branch = "v1", features = ["sqlite"] }
+ （改为 Tauri 2.x 对应的插件引用方式）
```

### 2. `src-tauri/src/main.rs`（及其他 Rust 源文件）

Tauri 2.x 的 `Builder` API 完全重写：

```rust
// Tauri 1.x
tauri::Builder::default()
    .plugin(tauri_plugin_single_instance::init(...))
    .invoke_handler(tauri::generate_handler![...])
    .run(tauri::generate_context!())

// Tauri 2.x
tauri::Builder::default()
    .plugin(tauri_plugin_single_instance::init(...))
    .plugin(tauri::plugin::store::Builder::default().build())  // store 变成内置插件
    .setup(|app| { ... })
    .run(tauri::generate_context!())
```

### 3. `src-tauri/tauri.conf.json`

2.x 配置结构有变更：
- `systemTray` 配置可能变化
- `bundle` 结构变化
- `updater` 结构变化
- 需对照 [Tauri 2.x 官方文档](https://v2.tauri.app/) 逐项检查

### 4. 前端代码

- `@tauri-apps/api` 从 v1 升级到 v2
- 检查所有 `window.__TAURI__` 调用是否有 API 变化
- 检查 `tauri.conf.json` 中 `build.distDir` 路径是否匹配

### 5. AUR PKGBUILD（`pot-translation-git`）

```diff
 depends=(
-   'webkit2gtk'
-   'libsoup'
+   'webkit2gtk-4.1'
+   'libsoup3'
    ...
 )
```

## 破坏性变更清单

| 变更 | 影响 |
|------|------|
| `#[tauri::command]` → `#[tauri::handler]` | 所有命令处理函数 |
| `WebviewWindow::builder` API 变化 | 窗口创建代码 |
| 插件系统 `init()` → `Builder` 模式 | 6 个插件都需要改 |
| `tauri::Manager` trait 方法变更 | 全局 shortcut、notification 等 |
| `PathResolver` / `State` API 变化 | 依赖注入方式 |
| `libsoup2` → `libsoup3` | HTTP 相关可能有 API 差异 |
| `gtk` 0.16 → 0.18 | 屏幕截图等 GTK 操作 |

## 升级步骤（按顺序）

### Step 1：用 `migrate-deps` 升级 Cargo.toml 版本号

```bash
cargo install migrate-deps
cd src-tauri
migrate-deps
```

这会交互式地列出可升级的依赖，重点升级：
- `tauri` → 2.x
- `tauri-build` → 2.x
- 所有 `tauri-plugin-*` → 对应 2.x 版本

> 升级 Tauri 到 2.x **自动**拉入 `wry 0.55` → `webkit2gtk 2.0.2` → `webkit2gtk-4.1`，不需要单独改 webkit2gtk 依赖。

### Step 2：`cargo check` 看编译错误

```bash
cargo check
```

此时编译肯定失败，因为 API 变了。把报错全部记录下来，按模块分类修复。

### Step 3：改 Rust 代码（按报错逐个修复）

优先级顺序：

1. **`main.rs` — Builder 重构**
   - `tauri::Builder` 链式调用方式变化
   - `.invoke_handler()` → `.setup()` / `.plugin()` 写法变化
   - 6 个插件（single-instance, autostart, fs-watch, store, log, sql）全部改引用方式

2. **命令处理函数**
   - `#[tauri::command]` → `#[tauri::handler]`
   - 参数签名可能变化（`AppHandle` vs `WebviewWindow`）

3. **Manager trait**
   - `app.state()` / `app.path()` 等 API 变化
   - `WebviewWindow` 方法变化

4. **屏幕截图 / GTK 相关**（linux-body feature）
   - `gtk` 0.16 → 0.18 API 差异
   - `screenshots` crate 是否需要更新

5. **其他 crate 版本冲突**
   - `libsoup2` → `libsoup3`：`reqwest` 后端可能自动切换
   - `font-kit`、`arboard`、`mouse_position` 等是否有 GTK 版本要求

### Step 4：改前端代码

1. `package.json` 中 `@tauri-apps/api` 升级到 v2
2. 全局搜索 `window.__TAURI__` 调用，对照 v2 API 文档检查
3. 检查 `pnpm-workspace.yaml` 和 Vite 配置是否兼容

### Step 5：改 `tauri.conf.json`

对照 [Tauri 2.x 配置文档](https://v2.tauri.app/reference/config/) 逐项检查：
- `systemTray` 结构
- `bundle` 结构
- `updater` 结构
- `build.distDir` 和 `devPath`

### Step 6：`cargo build --release` 确认编译通过

```bash
pnpm run build
```

### Step 7：运行测试

```bash
./target/release/pot
```

确认功能正常：窗口打开、翻译请求、系统托盘、快捷键、单例检查、SQL 存储、日志。

### Step 8：更新 AUR PKGBUILD

```diff
 depends=(
-   'webkit2gtk'
-   'libsoup'
+   'webkit2gtk-4.1'
+   'libsoup3'
    ...
 )
```

提交 PR 给 AUR maintainer。

## 临时方案（升级前可用）

```bash
# 方案 A：用预编译二进制
yay -S pot-translation-bin

# 方案 B：设置环境变量 + 从 AUR 编译 webkit2gtk
export WEBKIT_DISABLE_DMABUF_RENDERER=1
yay -S webkit2gtk  # 编译一次，之后缓存
yay -S pot-translation-git
```