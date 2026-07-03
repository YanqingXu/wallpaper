# Wallpaper Switcher Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Windows Tauri 2 desktop app that randomly switches the desktop wallpaper from three bundled images plus user-selected images kept only for the current app session.

**Architecture:** Vue 3 + Vite renders a compact control panel and invokes Tauri commands. Rust owns the wallpaper pool, validation, random selection, resource path resolution, and Windows `SystemParametersInfoW` call. Core Rust behavior is covered by unit tests that do not change the real desktop wallpaper.

**Tech Stack:** Vue 3.5.39, Vite 8.1.3, TypeScript 6.0.3, Tauri npm packages 2.11.1/2.11.4, Rust 1.96.1, Tauri Rust crates using the compatible `2` version line, Windows API via `windows-sys`, Tauri dialog plugin.

---

## File Structure

- Create `.gitignore`: ignore generated Node, Rust, and Tauri build outputs.
- Create `.cargo/config.toml`: use a crates.io mirror for this Windows/China network environment.
- Create `package.json`, `index.html`, `tsconfig.json`, `vite.config.ts`: Vue/Vite/Tauri frontend project.
- Create `src/main.ts`, `src/App.vue`, `src/styles.css`, `src/vite-env.d.ts`: frontend UI and command calls.
- Create `src-tauri/Cargo.toml`, `src-tauri/build.rs`, `src-tauri/tauri.conf.json`: Tauri/Rust project and bundler config.
- Create `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`: Tauri entrypoint and command registration.
- Create `src-tauri/src/wallpaper.rs`: testable wallpaper domain model, image validation, random pool selection.
- Create `src-tauri/src/platform.rs`: Windows wallpaper API boundary.
- Keep `res/1.png`, `res/2.png`, `res/3.png`: bundled wallpaper resources.

## Task 1: Project Scaffold

**Files:**
- Create: `.gitignore`
- Create: `.cargo/config.toml`
- Create: `package.json`
- Create: `index.html`
- Create: `tsconfig.json`
- Create: `vite.config.ts`
- Create: `src/main.ts`
- Create: `src/App.vue`
- Create: `src/styles.css`
- Create: `src/vite-env.d.ts`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create baseline scaffold files**

Create minimal Vue and Tauri files. `package.json` must include:

```json
{
  "name": "wallpaper-switcher",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite --host 127.0.0.1 --port 1420",
    "build": "vue-tsc --noEmit && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "2.11.1",
    "@tauri-apps/plugin-dialog": "2.7.1",
    "vue": "3.5.39"
  },
  "devDependencies": {
    "@tauri-apps/cli": "2.11.4",
    "@vitejs/plugin-vue": "6.0.7",
    "typescript": "6.0.3",
    "vite": "8.1.3",
    "vue-tsc": "3.3.6"
  }
}
```

`src-tauri/tauri.conf.json` must set `frontendDist` to `../dist`, `devUrl` to `http://127.0.0.1:1420`, and bundle `../res/1.png`, `../res/2.png`, `../res/3.png` as resources.

- [ ] **Step 2: Install frontend dependencies**

Run: `npm install`

Expected: `package-lock.json` is created and dependency install exits with code 0.

- [ ] **Step 3: Verify scaffold build fails only because Rust commands are not implemented yet**

Run: `npm run build`

Expected: Vue build succeeds for the minimal placeholder UI. If it fails, fix scaffold syntax before moving on.

- [ ] **Step 4: Commit scaffold**

```powershell
git add .gitignore .cargo package.json package-lock.json index.html tsconfig.json vite.config.ts src src-tauri res
git commit -m "chore: scaffold tauri vue app"
```

## Task 2: Rust Wallpaper Domain With TDD

**Files:**
- Create: `src-tauri/src/wallpaper.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/wallpaper.rs`

- [ ] **Step 1: Write failing Rust tests**

Add tests covering:

```rust
#[test]
fn supported_image_extensions_are_case_insensitive() {
    assert!(is_supported_image_path(Path::new("scene.PNG")));
    assert!(is_supported_image_path(Path::new("photo.jpeg")));
    assert!(is_supported_image_path(Path::new("wall.BMP")));
    assert!(!is_supported_image_path(Path::new("notes.txt")));
}

#[test]
fn pool_combines_bundled_and_user_wallpapers() {
    let mut pool = WallpaperPool::new(vec![PathBuf::from("one.png"), PathBuf::from("two.png")]);
    pool.add_existing_user_path_for_test(PathBuf::from("user.jpg"));
    let all = pool.all();
    assert_eq!(all.len(), 3);
    assert!(all.iter().any(|item| item.source == WallpaperSource::Bundled));
    assert!(all.iter().any(|item| item.source == WallpaperSource::User));
}

#[test]
fn random_from_empty_pool_returns_error() {
    let pool = WallpaperPool::new(Vec::new());
    let err = pool.random().unwrap_err();
    assert_eq!(err, WallpaperError::EmptyPool);
}
```

- [ ] **Step 2: Run tests and verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml wallpaper --lib`

Expected: FAIL because `wallpaper` module, `WallpaperPool`, and validation functions do not exist.

- [ ] **Step 3: Implement minimal domain model**

Implement:

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum WallpaperSource {
    Bundled,
    User,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct WallpaperItem {
    pub id: String,
    pub label: String,
    pub path: String,
    pub source: WallpaperSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WallpaperError {
    EmptyPool,
    FileMissing,
    UnsupportedImageType,
}
```

`WallpaperPool` stores bundled paths and session-only user paths. `is_supported_image_path` accepts `png`, `jpg`, `jpeg`, and `bmp` case-insensitively.

- [ ] **Step 4: Run tests and verify GREEN**

Run: `cargo test --manifest-path src-tauri/Cargo.toml wallpaper --lib`

Expected: all wallpaper domain tests pass.

- [ ] **Step 5: Commit domain model**

```powershell
git add src-tauri/src/lib.rs src-tauri/src/wallpaper.rs src-tauri/Cargo.toml
git commit -m "feat: add wallpaper pool domain"
```

## Task 3: Tauri Commands and Windows API Boundary

**Files:**
- Create: `src-tauri/src/platform.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/wallpaper.rs`
- Test: `src-tauri/src/wallpaper.rs`

- [ ] **Step 1: Write failing command-support tests**

Add tests for `add_user_wallpaper` rejecting missing files and unsupported extensions by calling the domain method that backs the Tauri command:

```rust
#[test]
fn add_user_wallpaper_rejects_missing_file() {
    let mut pool = WallpaperPool::new(Vec::new());
    let err = pool.add_user_wallpaper(Path::new("missing.png")).unwrap_err();
    assert_eq!(err, WallpaperError::FileMissing);
}

#[test]
fn add_user_wallpaper_rejects_unsupported_extension() {
    let file = tempfile::NamedTempFile::new().unwrap();
    let bad_path = file.path().with_extension("gif");
    std::fs::write(&bad_path, b"fake").unwrap();
    let mut pool = WallpaperPool::new(Vec::new());
    let err = pool.add_user_wallpaper(&bad_path).unwrap_err();
    assert_eq!(err, WallpaperError::UnsupportedImageType);
    std::fs::remove_file(bad_path).unwrap();
}
```

- [ ] **Step 2: Run tests and verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml wallpaper --lib`

Expected: FAIL because `add_user_wallpaper` validation is not implemented.

- [ ] **Step 3: Implement validation and command wrappers**

Implement Tauri state:

```rust
pub struct AppState {
    pub pool: std::sync::Mutex<WallpaperPool>,
}
```

Expose commands:

```rust
#[tauri::command]
fn list_wallpapers(state: tauri::State<AppState>) -> Vec<WallpaperItem>;

#[tauri::command]
fn add_user_wallpaper(path: String, state: tauri::State<AppState>) -> Result<Vec<WallpaperItem>, String>;

#[tauri::command]
fn set_wallpaper(path: String) -> Result<(), String>;

#[tauri::command]
fn set_random_wallpaper(state: tauri::State<AppState>) -> Result<WallpaperItem, String>;
```

`platform::set_desktop_wallpaper` uses `SystemParametersInfoW` on Windows and returns a clear unsupported-platform error on non-Windows.

- [ ] **Step 4: Run Rust tests and build**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: all Rust tests pass.

Run: `cargo build --manifest-path src-tauri/Cargo.toml`

Expected: Rust project builds.

- [ ] **Step 5: Commit Tauri command layer**

```powershell
git add src-tauri/src src-tauri/Cargo.toml
git commit -m "feat: expose wallpaper tauri commands"
```

## Task 4: Vue Control Panel

**Files:**
- Modify: `src/App.vue`
- Modify: `src/styles.css`
- Modify: `src/main.ts`

- [ ] **Step 1: Implement command client and UI state**

`App.vue` must:

- Call `list_wallpapers` on mount.
- Call Tauri dialog `open` with `png`, `jpg`, `jpeg`, and `bmp` filters.
- Call `add_user_wallpaper` for selected files.
- Call `set_random_wallpaper` from the main button.
- Call `set_wallpaper` when a wallpaper tile is clicked.
- Show count, current status, loading state, bundled/user badges, and image previews.

- [ ] **Step 2: Style the app**

Use a compact app layout, stable button sizes, clear focus states, and non-overlapping responsive grid. Avoid nested cards and oversized landing-page treatment.

- [ ] **Step 3: Verify frontend build**

Run: `npm run build`

Expected: TypeScript and Vite build pass with exit code 0.

- [ ] **Step 4: Commit frontend**

```powershell
git add src
git commit -m "feat: build wallpaper switcher ui"
```

## Task 5: Package and Final Verification

**Files:**
- Modify: the exact source or config file named by a failed verification output, then rerun the failed command before continuing.

- [ ] **Step 1: Run full Rust tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: all Rust tests pass.

- [ ] **Step 2: Run frontend build**

Run: `npm run build`

Expected: Vue/TypeScript/Vite build passes.

- [ ] **Step 3: Run Tauri build**

Run: `npm run tauri build`

Expected: Tauri creates Windows release artifacts under `src-tauri/target/release/bundle/`.

- [ ] **Step 4: Locate exe artifacts**

Run:

```powershell
Get-ChildItem -Recurse src-tauri\target\release\bundle -Include *.exe | Select-Object FullName,Length
```

Expected: at least one `.exe` path is printed.

- [ ] **Step 5: Final status**

Run:

```powershell
git status --short --branch
```

Expected: only intentional uncommitted build artifacts are ignored; source tree is clean.
