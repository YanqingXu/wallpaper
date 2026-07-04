# 扩展与维护

这份文档记录常见改动应该从哪里下手，以及需要同步检查哪些文件。

## 新增内置壁纸

1. 把图片放入 `res/` 目录。
2. 在 `src-tauri/src/lib.rs` 的 `BUNDLED_WALLPAPERS` 中新增一项。
3. 如果仍保留 `tauri.conf.json` 的 `bundle.resources` 配置，也同步加入新图片。
4. 运行应用，确认前端统计中的“内置”数量增加。

示例：

```rust
BundledWallpaper {
    file_name: "4.png",
    bytes: include_bytes!("../../res/4.png"),
},
```

注意：仅把图片放到 `res/` 目录不会自动进入内置壁纸列表。

## 支持更多图片格式

需要同时改前端和后端：

- 前端：`src/App.vue` 中 `open()` 的 `filters.extensions` 增加新扩展名。
- 后端：`src-tauri/src/wallpaper.rs` 中 `is_supported_image_path()` 增加新扩展名。
- 测试：为新扩展名补充单元测试。

后端校验是最终约束，不能只改前端过滤器。

## 持久化用户壁纸

当前 `user_paths` 只存在内存里，应用关闭后会丢失。要持久化可以考虑：

1. 在 Tauri app data 目录保存一个 JSON 文件。
2. 启动时读取 JSON，过滤不存在或不支持的路径。
3. `add_user_wallpaper()` 成功后写回 JSON。
4. 增加删除本地壁纸的命令，避免列表只能增加不能减少。

推荐把持久化逻辑放在新的 Rust 模块中，例如 `storage.rs`，让 `wallpaper.rs` 保持纯业务模型。

## 增加删除壁纸功能

前端需要：

- 在壁纸卡片上增加删除按钮。
- 区分内置和用户壁纸，通常只允许删除用户壁纸。
- 删除后刷新 `wallpapers`。

后端需要：

- 在 `WallpaperPool` 增加按路径或 id 删除用户壁纸的方法。
- 新增 Tauri command，例如 `remove_user_wallpaper`。
- 对路径不存在、不是用户壁纸等情况返回清晰错误。

## 增加跨平台支持

平台相关逻辑集中在 `src-tauri/src/platform.rs`。新增平台支持时优先保持这个边界：

- Windows：当前已有 `SystemParametersInfoW` 实现。
- macOS：可考虑调用系统脚本或使用原生 API。
- Linux：需要根据桌面环境区分 GNOME、KDE、XFCE 等。

前端不应该直接判断操作系统。后端命令失败时返回中文错误，前端负责展示即可。

## 调整前端界面

主要修改点：

- `src/App.vue`：新增状态、按钮和交互函数。
- `src/styles.css`：调整布局、颜色、响应式行为。
- `src-tauri/capabilities/default.json`：如果新增插件或文件能力，可能需要补权限。

新增 Tauri command 后，要在 `src-tauri/src/lib.rs` 的 `tauri::generate_handler![]` 中注册，否则前端 `invoke()` 会找不到命令。

## 维护检查清单

代码改动后建议按改动范围运行：

- 前端逻辑或类型变更：`npm run build`
- Rust 业务逻辑变更：`cd src-tauri && cargo test`
- Tauri 命令或配置变更：`npm run tauri dev` 手动验证窗口行为
- 壁纸资源变更：确认内置壁纸数量、图片预览和实际设置壁纸都正常

