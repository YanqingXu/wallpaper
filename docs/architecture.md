# 项目架构

## 总体结构

Wallpaper Switcher 分为两层：

- 前端层：`src/` 下的 Vue 应用，负责页面渲染、按钮交互、文件选择和展示操作结果。
- 后端层：`src-tauri/` 下的 Rust/Tauri 应用，负责维护壁纸数据、校验图片路径、写入内置资源并调用系统 API。

前端和后端通过 Tauri command 通信。前端使用 `invoke()` 调用 Rust 中标记为 `#[tauri::command]` 的函数。

## 启动流程

1. `src-tauri/src/main.rs` 调用 `wallpaper_switcher_lib::run()`。
2. `src-tauri/src/lib.rs` 构建 Tauri 应用。
3. Tauri 初始化 dialog 插件，让前端可以打开系统文件选择器。
4. `setup()` 阶段调用 `bundled_wallpaper_paths()`。
5. `bundled_wallpaper_paths()` 选择应用缓存目录下的 `bundled-wallpapers` 子目录。
6. `materialize_bundled_files()` 把 `include_bytes!` 编译进程序的内置图片写入缓存目录。
7. 应用创建 `AppState`，其中包含一个由 `Mutex` 保护的 `WallpaperPool`。
8. `invoke_handler` 注册前端可调用的命令。
9. 前端加载后，`App.vue` 的 `onMounted()` 调用 `refreshWallpapers()` 获取列表。

## 主要运行链路

### 列出壁纸

1. 前端调用 `invoke<WallpaperItem[]>('list_wallpapers')`。
2. 后端 `list_wallpapers()` 锁定 `AppState.pool`。
3. `WallpaperPool::all()` 合并内置壁纸和用户壁纸。
4. 后端返回序列化后的 `WallpaperItem` 数组。
5. 前端把结果写入 `wallpapers`，网格自动重新渲染。

### 添加本地壁纸

1. 前端通过 `open()` 打开系统图片选择器。
2. 用户选择图片后，前端调用 `add_user_wallpaper` 并传入路径。
3. 后端 `WallpaperPool::add_user_wallpaper()` 校验文件存在和扩展名。
4. 如果这个路径尚未加入本次会话的用户壁纸列表，就追加到 `user_paths`。
5. 后端返回最新壁纸列表。

用户壁纸目前只保存在内存中，应用重启后不会自动恢复。

### 设置指定壁纸

1. 用户点击某个壁纸卡片。
2. 前端调用 `set_wallpaper` 并传入图片路径。
3. 后端先调用 `validate_existing_image_path()` 做文件存在和格式校验。
4. 后端调用 `platform::set_desktop_wallpaper()`。
5. Windows 平台使用 Win32 API 设置桌面壁纸。

### 随机切换壁纸

1. 用户点击“随机切换”。
2. 前端调用 `set_random_wallpaper`。
3. 后端从 `WallpaperPool::all()` 中随机选一项。
4. 后端调用平台层设置壁纸。
5. 设置成功后返回被选中的 `WallpaperItem`。
6. 前端更新 `current` 和状态文本。

## 数据边界

前端只处理适合展示的数据结构：

```ts
interface WallpaperItem {
  id: string;
  label: string;
  path: string;
  source: 'bundled' | 'user';
}
```

Rust 端的 `WallpaperItem` 使用 `serde` 序列化，并通过 `#[serde(rename_all = "camelCase")]` 保证字段名适合 TypeScript 使用。

## 并发与状态

后端的全局状态是：

```rust
pub struct AppState {
    pool: Mutex<WallpaperPool>,
}
```

`Mutex` 的作用是保护壁纸池，避免多个命令同时修改用户壁纸列表。前端也有 `busy` 状态，用来避免用户连续点击触发重叠操作。

