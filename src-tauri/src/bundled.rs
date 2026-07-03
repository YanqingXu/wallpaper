use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy)]
pub struct BundledWallpaper {
    pub file_name: &'static str,
    pub bytes: &'static [u8],
}

pub fn materialize_bundled_files(
    target_dir: &Path,
    entries: &[BundledWallpaper],
) -> Result<Vec<PathBuf>, String> {
    fs::create_dir_all(target_dir).map_err(|error| {
        format!(
            "创建内置壁纸目录失败 {}：{}",
            target_dir.display(),
            error
        )
    })?;

    entries
        .iter()
        .map(|entry| {
            let path = target_dir.join(entry.file_name);
            let should_write = fs::read(&path)
                .map(|existing| existing != entry.bytes)
                .unwrap_or(true);

            if should_write {
                fs::write(&path, entry.bytes)
                    .map_err(|error| format!("写入内置壁纸失败 {}：{}", path.display(), error))?;
            }

            Ok(path)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn materialize_bundled_files_writes_entries_to_target_dir() {
        let dir = tempfile::tempdir().unwrap();
        let entries = [
            BundledWallpaper {
                file_name: "one.png",
                bytes: b"first",
            },
            BundledWallpaper {
                file_name: "two.jpg",
                bytes: b"second",
            },
        ];

        let paths = materialize_bundled_files(dir.path(), &entries).unwrap();

        assert_eq!(paths.len(), 2);
        assert_eq!(std::fs::read(&paths[0]).unwrap(), b"first");
        assert_eq!(std::fs::read(&paths[1]).unwrap(), b"second");
    }

    #[test]
    fn materialize_bundled_files_rewrites_stale_file() {
        let dir = tempfile::tempdir().unwrap();
        let stale_path = dir.path().join("one.png");
        std::fs::write(&stale_path, b"stale").unwrap();
        let entries = [BundledWallpaper {
            file_name: "one.png",
            bytes: b"fresh",
        }];

        let paths = materialize_bundled_files(dir.path(), &entries).unwrap();

        assert_eq!(paths, vec![stale_path.clone()]);
        assert_eq!(std::fs::read(stale_path).unwrap(), b"fresh");
    }
}
