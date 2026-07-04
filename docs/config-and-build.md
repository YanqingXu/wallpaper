# 配置与构建

这个项目由前端 npm 工程和 Rust/Tauri 工程共同组成。前端负责打包 Web UI，Tauri 负责桌面窗口、权限和后端命令。

## package.json

`package.json` 定义前端和 Tauri 常用脚本：

| 命令 | 作用 |
| --- | --- |
| `npm run dev` | 启动 Vite 开发服务器，监听 `127.0.0.1:1420`。 |
| `npm run build` | 先运行 `vue-tsc --noEmit` 类型检查，再执行 Vite 构建。 |
| `npm run preview` | 预览 Vite 构建产物。 |
| `npm run tauri` | 调用 Tauri CLI。 |

主要依赖：

- `vue`：前端框架。
- `@tauri-apps/api`：前端调用 Tauri API。
- `@tauri-apps/plugin-dialog`：系统文件选择器。
- `@lucide/vue`：界面图标。

## Vite 配置

`vite.config.ts` 使用 Vue 插件，并固定开发服务器：

```ts
server: {
  host: '127.0.0.1',
  port: 1420,
  strictPort: true,
}
```

`strictPort: true` 与 Tauri 的 `devUrl` 配合，确保桌面壳加载的是预期端口。

## Tauri 配置

`src-tauri/tauri.conf.json` 配置桌面应用。

### build

| 字段 | 说明 |
| --- | --- |
| `beforeDevCommand` | Tauri dev 前启动 Vite。 |
| `devUrl` | 开发模式窗口加载的前端地址。 |
| `beforeBuildCommand` | Tauri build 前执行前端构建。 |
| `frontendDist` | 生产模式加载的前端产物目录。 |

### app.windows

当前只定义一个主窗口：

- 标题：`Wallpaper Switcher`
- 初始尺寸：`860 x 620`
- 最小尺寸：`680 x 520`
- 可调整大小：`true`

### security.assetProtocol

项目启用了 Tauri asset protocol，并允许前端加载这些路径下的本地图片：

- `$APPCACHE/**`
- `$HOME/**`
- `$PICTURE/**`
- `$DOWNLOAD/**`
- `$DESKTOP/**`
- `$DOCUMENT/**`
- `$TEMP/**`

这与前端的 `convertFileSrc(path)` 配套使用。

### bundle.resources

配置里列出了 `../res/1.png` 到 `../res/3.png`。当前运行时代码主要依赖 `include_bytes!` 嵌入内置壁纸；如果未来改成从 bundle resource 读取，这部分配置会变得更关键。

## 权限配置

`src-tauri/capabilities/default.json` 声明默认窗口权限：

- `core:default`
- `dialog:default`

`dialog:default` 是前端能够打开文件选择器的关键权限。

## Rust 依赖

`src-tauri/Cargo.toml` 中的核心依赖：

| 依赖 | 作用 |
| --- | --- |
| `tauri` | 桌面应用框架。 |
| `tauri-plugin-dialog` | Tauri dialog 插件后端部分。 |
| `serde` | 序列化返回给前端的数据结构。 |
| `rand` | 随机选择壁纸。 |
| `windows-sys` | 调用 Windows 桌面壁纸 API。 |
| `tempfile` | 单元测试中创建临时文件和目录。 |

## 常用开发命令

```bash
npm install
npm run tauri dev
```

前端类型检查和构建：

```bash
npm run build
```

Rust 单元测试：

```bash
cd src-tauri
cargo test
```

