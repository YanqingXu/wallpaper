# 后端代码说明

后端代码位于 `src-tauri/src/`。它既是 Tauri 应用的宿主，也是壁纸业务逻辑的主要实现位置。

## 模块划分

| 文件 | 作用 |
| --- | --- |
| `main.rs` | 二进制入口，调用库函数启动应用。 |
| `lib.rs` | Tauri 初始化、状态管理、命令定义和命令注册。 |
| `wallpaper.rs` | 壁纸数据模型、壁纸池、图片校验和单元测试。 |
| `bundled.rs` | 内置壁纸写入缓存目录。 |
| `platform.rs` | 调用操作系统接口设置桌面壁纸。 |

## lib.rs

`lib.rs` 是后端的主装配层。

### 内置壁纸声明

`BUNDLED_WALLPAPERS` 使用 `include_bytes!` 把图片编译进程序：

```rust
const BUNDLED_WALLPAPERS: &[BundledWallpaper] = &[
    BundledWallpaper {
        file_name: "1.png",
        bytes: include_bytes!("../../res/1.png"),
    },
];
```

当前代码注册了 `res/1.png`、`res/2.png`、`res/3.png`。如果 `res/` 目录中新增图片，需要同步把它加入这个常量，否则不会成为内置壁纸。

### 应用状态

`AppState` 保存一个 `Mutex<WallpaperPool>`：

```rust
pub struct AppState {
    pool: Mutex<WallpaperPool>,
}
```

所有读写壁纸池的 Tauri command 都会先锁定这个状态。

### Tauri 命令

| 命令 | 返回 | 说明 |
| --- | --- | --- |
| `list_wallpapers` | `Vec<WallpaperItem>` | 返回全部内置和用户壁纸。 |
| `add_user_wallpaper` | `Vec<WallpaperItem>` | 校验并加入用户选择的本地图片。 |
| `set_wallpaper` | `()` | 设置指定路径为桌面壁纸。 |
| `set_random_wallpaper` | `WallpaperItem` | 随机选一张并设置为桌面壁纸。 |

命令内部把业务错误转换为中文用户提示字符串，前端直接显示这些字符串。

## wallpaper.rs

`wallpaper.rs` 是壁纸业务模型。

### 数据类型

`WallpaperSource` 表示壁纸来源：

- `Bundled`：内置壁纸。
- `User`：用户本次会话选择的本地图片。

`WallpaperItem` 是返回给前端的数据：

- `id`：前端列表 key。
- `label`：从文件名 stem 推导出的展示名。
- `path`：图片绝对或本地路径字符串。
- `source`：壁纸来源。

### WallpaperPool

`WallpaperPool` 保存两组路径：

- `bundled_paths`：启动时从内置资源 materialize 后得到。
- `user_paths`：用户在本次运行中选择的图片。

主要方法：

- `new()`：用内置路径创建池。
- `all()`：把内置和用户路径合并成 `WallpaperItem`。
- `random()`：使用随机数生成器选择一张。
- `random_with_rng()`：方便测试注入随机数。
- `add_user_wallpaper()`：校验并追加用户图片。

### 图片校验

`is_supported_image_path()` 只检查扩展名，支持：

- `png`
- `jpg`
- `jpeg`
- `bmp`

`validate_existing_image_path()` 先检查路径是否存在，再检查扩展名是否支持。

## bundled.rs

`bundled.rs` 负责把编译进程序的内置图片写到磁盘缓存目录。这样前端可以通过本地路径展示图片，平台 API 也能拿到真实文件路径。

`materialize_bundled_files()` 的逻辑是：

1. 确保目标目录存在。
2. 遍历所有 `BundledWallpaper`。
3. 如果目标文件不存在，写入图片 bytes。
4. 如果目标文件存在但内容不同，重写文件。
5. 返回所有写入目标路径。

该模块已有单元测试覆盖写入和重写过期文件。

## platform.rs

`platform.rs` 把系统相关代码隔离在单独模块中。

Windows 平台实现使用 `windows-sys` 调用：

- `SystemParametersInfoW`
- `SPI_SETDESKWALLPAPER`
- `SPIF_UPDATEINIFILE`
- `SPIF_SENDCHANGE`

这会设置桌面壁纸，并通知系统配置发生变化。

非 Windows 平台当前返回错误：

```rust
Err("当前平台不支持设置 Windows 桌面壁纸".to_string())
```

如果后续要支持 macOS 或 Linux，应该在这个模块增加对应的 `#[cfg(target_os = "...")]` 实现。

## 测试覆盖

当前 Rust 单元测试覆盖了：

- 图片扩展名大小写不敏感。
- 壁纸池合并内置和用户壁纸。
- 空池随机选择返回错误。
- 添加不存在文件返回错误。
- 添加不支持格式返回错误。
- 添加支持格式成功。
- 内置资源写入目标目录。
- 内置资源内容过期时会重写。

