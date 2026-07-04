# 前端代码说明

前端代码位于 `src/`，核心文件是 `src/App.vue`。这个项目没有路由和复杂状态库，所有界面状态都集中在单文件组件中。

## 入口文件

`src/main.ts` 做三件事：

1. 引入 Vue 的 `createApp`。
2. 引入根组件 `App.vue`。
3. 引入全局样式 `styles.css` 并挂载应用。

```ts
createApp(App).mount('#app');
```

## App.vue 的职责

`App.vue` 包含三类内容：

- TypeScript 逻辑：壁纸类型、响应式状态、Tauri 调用和操作函数。
- Template：顶部统计、操作按钮、状态提示、当前壁纸和壁纸网格。
- 组件依赖：Tauri API、dialog 插件、Lucide 图标和 Vue composition API。

## 前端状态

| 状态 | 类型 | 作用 |
| --- | --- | --- |
| `wallpapers` | `WallpaperItem[]` | 当前可展示的壁纸列表。 |
| `current` | `WallpaperItem \| null` | 最近成功设置的壁纸。 |
| `status` | `string` | 页面状态栏显示的文字。 |
| `statusKind` | `'info' \| 'success' \| 'error'` | 控制状态栏样式和图标。 |
| `busy` | `boolean` | 阻止操作重复触发，并控制按钮禁用状态。 |

`bundledCount` 和 `userCount` 是计算属性，分别统计内置壁纸和本地壁纸数量。

## 与 Tauri 后端通信

前端使用 `@tauri-apps/api/core` 的 `invoke()` 调用后端命令。

| 前端函数 | 调用的命令 | 后端行为 |
| --- | --- | --- |
| `refreshWallpapers()` | `list_wallpapers` | 获取全部壁纸。 |
| `chooseLocalWallpaper()` | `add_user_wallpaper` | 添加本地图片路径并返回最新列表。 |
| `setRandomWallpaper()` | `set_random_wallpaper` | 随机选择并设置壁纸。 |
| `setSpecificWallpaper(item)` | `set_wallpaper` | 设置用户点击的指定壁纸。 |

所有需要显示图片的本地路径都经过 `convertFileSrc(path)` 转换。Tauri 的 asset protocol 会把本地文件路径转成前端可以加载的地址。

## 文件选择

`chooseLocalWallpaper()` 使用 `@tauri-apps/plugin-dialog` 的 `open()`：

- `multiple: false`：只允许选择一张图。
- `directory: false`：选择文件而不是目录。
- `filters`：只显示 `png`、`jpg`、`jpeg`、`bmp`。

前端过滤是用户体验层面的限制，真正的安全校验仍由 Rust 后端的 `validate_existing_image_path()` 完成。

## 操作封装

`runAction()` 是前端的通用异步操作包装器：

1. 如果当前 `busy` 为 `true`，直接返回。
2. 开始操作前设为忙碌状态。
3. 捕获异常并显示错误信息。
4. 操作结束后恢复按钮可用。

这让“选择图片”、“随机切换”和“设置指定壁纸”共享同一套错误处理和防重复点击逻辑。

## 视觉结构

模板分成五个主要区域：

- `.topbar`：标题和内置/本地数量。
- `.controls`：随机切换、选择图片、刷新三个操作按钮。
- `.status-line`：操作反馈。
- `.current-strip`：最近设置成功的壁纸。
- `.wallpaper-grid`：壁纸卡片列表。

`src/styles.css` 定义全局字体、响应式布局、按钮状态、状态提示颜色和壁纸网格。移动端在 `700px` 以下把顶部和按钮布局切成单列。

