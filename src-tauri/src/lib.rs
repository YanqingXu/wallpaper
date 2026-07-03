pub mod bundled;
mod platform;
pub mod wallpaper;

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, Runtime, State};
use bundled::{materialize_bundled_files, BundledWallpaper};
use wallpaper::{validate_existing_image_path, WallpaperItem, WallpaperPool};

const BUNDLED_WALLPAPERS: &[BundledWallpaper] = &[
    BundledWallpaper {
        file_name: "1.png",
        bytes: include_bytes!("../../res/1.png"),
    },
    BundledWallpaper {
        file_name: "2.png",
        bytes: include_bytes!("../../res/2.png"),
    },
    BundledWallpaper {
        file_name: "3.png",
        bytes: include_bytes!("../../res/3.png"),
    },
];

pub struct AppState {
    pool: Mutex<WallpaperPool>,
}

#[tauri::command]
fn list_wallpapers(state: State<'_, AppState>) -> Result<Vec<WallpaperItem>, String> {
    let pool = state
        .pool
        .lock()
        .map_err(|_| "壁纸状态锁定失败".to_string())?;
    Ok(pool.all())
}

#[tauri::command]
fn add_user_wallpaper(
    path: String,
    state: State<'_, AppState>,
) -> Result<Vec<WallpaperItem>, String> {
    let mut pool = state
        .pool
        .lock()
        .map_err(|_| "壁纸状态锁定失败".to_string())?;

    pool.add_user_wallpaper(Path::new(&path))
        .map_err(|error| error.user_message().to_string())
}

#[tauri::command]
fn set_wallpaper(path: String) -> Result<(), String> {
    let path = PathBuf::from(path);
    validate_existing_image_path(&path).map_err(|error| error.user_message().to_string())?;
    platform::set_desktop_wallpaper(&path)
}

#[tauri::command]
fn set_random_wallpaper(state: State<'_, AppState>) -> Result<WallpaperItem, String> {
    let wallpaper = {
        let pool = state
            .pool
            .lock()
            .map_err(|_| "壁纸状态锁定失败".to_string())?;
        pool.random()
            .map_err(|error| error.user_message().to_string())?
    };

    platform::set_desktop_wallpaper(Path::new(&wallpaper.path))?;
    Ok(wallpaper)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let bundled_paths = bundled_wallpaper_paths(app.handle()).map_err(|error| {
                std::io::Error::new(std::io::ErrorKind::Other, error)
            })?;
            app.manage(AppState {
                pool: Mutex::new(WallpaperPool::new(bundled_paths)),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_wallpapers,
            add_user_wallpaper,
            set_wallpaper,
            set_random_wallpaper
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn bundled_wallpaper_paths<R: Runtime>(app: &AppHandle<R>) -> Result<Vec<PathBuf>, String> {
    let target_dir = app
        .path()
        .app_cache_dir()
        .unwrap_or_else(|_| std::env::temp_dir().join("wallpaper-switcher"))
        .join("bundled-wallpapers");

    materialize_bundled_files(&target_dir, BUNDLED_WALLPAPERS)
}
