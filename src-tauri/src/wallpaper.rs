use rand::Rng;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WallpaperSource {
    Bundled,
    User,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
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

impl WallpaperError {
    pub fn user_message(&self) -> &'static str {
        match self {
            WallpaperError::EmptyPool => "没有可用壁纸",
            WallpaperError::FileMissing => "图片文件不存在",
            WallpaperError::UnsupportedImageType => "仅支持 png、jpg、jpeg、bmp",
        }
    }
}

#[derive(Debug, Clone)]
pub struct WallpaperPool {
    bundled_paths: Vec<PathBuf>,
    user_paths: Vec<PathBuf>,
}

impl WallpaperPool {
    pub fn new(bundled_paths: Vec<PathBuf>) -> Self {
        Self {
            bundled_paths,
            user_paths: Vec::new(),
        }
    }

    pub fn all(&self) -> Vec<WallpaperItem> {
        let bundled = self
            .bundled_paths
            .iter()
            .enumerate()
            .map(|(index, path)| wallpaper_item(index, path, WallpaperSource::Bundled));
        let users = self
            .user_paths
            .iter()
            .enumerate()
            .map(|(index, path)| wallpaper_item(index, path, WallpaperSource::User));

        bundled.chain(users).collect()
    }

    pub fn random(&self) -> Result<WallpaperItem, WallpaperError> {
        let mut rng = rand::rng();
        self.random_with_rng(&mut rng)
    }

    pub fn random_with_rng<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
    ) -> Result<WallpaperItem, WallpaperError> {
        let items = self.all();
        if items.is_empty() {
            return Err(WallpaperError::EmptyPool);
        }

        let index = rng.random_range(0..items.len());
        Ok(items[index].clone())
    }

    pub fn add_user_wallpaper(&mut self, path: &Path) -> Result<Vec<WallpaperItem>, WallpaperError> {
        validate_existing_image_path(path)?;
        let path = path.to_path_buf();

        if !self.user_paths.iter().any(|existing| existing == &path) {
            self.user_paths.push(path);
        }

        Ok(self.all())
    }

    #[cfg(test)]
    fn add_existing_user_path_for_test(&mut self, path: PathBuf) {
        self.user_paths.push(path);
    }
}

pub fn is_supported_image_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "bmp"
            )
        })
        .unwrap_or(false)
}

pub fn validate_existing_image_path(path: &Path) -> Result<(), WallpaperError> {
    if !path.exists() {
        return Err(WallpaperError::FileMissing);
    }

    if !is_supported_image_path(path) {
        return Err(WallpaperError::UnsupportedImageType);
    }

    Ok(())
}

fn wallpaper_item(index: usize, path: &Path, source: WallpaperSource) -> WallpaperItem {
    let prefix = match source {
        WallpaperSource::Bundled => "bundled",
        WallpaperSource::User => "user",
    };
    let label = path
        .file_stem()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("wallpaper")
        .to_string();

    WallpaperItem {
        id: format!("{prefix}-{index}"),
        label,
        path: path.to_string_lossy().into_owned(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

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

    #[test]
    fn add_user_wallpaper_rejects_missing_file() {
        let mut pool = WallpaperPool::new(Vec::new());

        let err = pool
            .add_user_wallpaper(Path::new("missing.png"))
            .unwrap_err();

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

    #[test]
    fn add_user_wallpaper_accepts_supported_file_for_session_pool() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let image_path = file.path().with_extension("jpg");
        std::fs::write(&image_path, b"fake").unwrap();
        let mut pool = WallpaperPool::new(Vec::new());

        let all = pool.add_user_wallpaper(&image_path).unwrap();

        assert_eq!(all.len(), 1);
        assert_eq!(all[0].source, WallpaperSource::User);
        assert_eq!(all[0].path, image_path.to_string_lossy());
        std::fs::remove_file(image_path).unwrap();
    }
}
