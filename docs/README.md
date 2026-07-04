# Wallpaper Switcher 文档

这个目录记录当前项目代码的结构、运行流程和常见维护方式。项目是一个基于 Vue 3 + Vite + Tauri 2 的桌面壁纸切换器，前端负责交互界面，Rust 后端负责壁纸列表管理、图片校验和调用系统 API 设置桌面壁纸。

## 文档索引

- [项目架构](./architecture.md)：整体模块关系、启动流程和用户操作链路。
- [前端代码说明](./frontend.md)：`src/App.vue`、状态管理、Tauri 命令调用和样式结构。
- [后端代码说明](./backend.md)：Rust 模块、Tauri 命令、壁纸池、内置资源和平台调用。
- [配置与构建](./config-and-build.md)：`package.json`、Vite、Tauri 配置、权限和常用命令。
- [扩展与维护](./extending.md)：新增内置壁纸、支持更多格式、持久化用户壁纸和跨平台扩展。

## 代码入口速览

| 路径 | 作用 |
| --- | --- |
| `src/main.ts` | 创建 Vue 应用并挂载 `App.vue`。 |
| `src/App.vue` | 主界面和所有前端交互逻辑。 |
| `src/styles.css` | 全局样式、布局、按钮、状态栏和壁纸网格。 |
| `src-tauri/src/main.rs` | Tauri 二进制入口，调用库中的 `run()`。 |
| `src-tauri/src/lib.rs` | Tauri 应用初始化、全局状态、命令注册。 |
| `src-tauri/src/wallpaper.rs` | 壁纸数据模型、壁纸池、图片格式校验和单元测试。 |
| `src-tauri/src/bundled.rs` | 把编译进程序的内置壁纸写入缓存目录。 |
| `src-tauri/src/platform.rs` | Windows 桌面壁纸设置实现和非 Windows fallback。 |
| `src-tauri/tauri.conf.json` | Tauri 窗口、开发命令、资源和安全配置。 |
| `src-tauri/capabilities/default.json` | Tauri 权限声明。 |

## 当前能力

- 启动时加载内置壁纸。
- 在前端显示内置壁纸和本次会话选择的本地壁纸。
- 支持选择本地 `png`、`jpg`、`jpeg`、`bmp` 图片。
- 支持随机切换壁纸。
- 支持点击指定壁纸后设置为桌面壁纸。
- Windows 下通过 `SystemParametersInfoW` 设置桌面壁纸。

