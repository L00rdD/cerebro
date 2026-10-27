use std::path::{Component, Path, PathBuf};

/// A file of the built UI (`dist/`): page, script, sprites…
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    pub bytes: Vec<u8>,
    pub mime_type: String,
}

/// Where the UI files come from: embedded in the app, or a folder on disk.
pub trait Assets: Send + Sync + 'static {
    /// `path` is relative (`index.html`, `characters/storm.png`) and never contains `..`.
    fn get(&self, path: &str) -> Option<Asset>;
}

/// Serves a folder, typically the `dist/` built by `npm run build:ui`.
pub struct DirAssets {
    root: PathBuf,
}

impl DirAssets {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl Assets for DirAssets {
    fn get(&self, path: &str) -> Option<Asset> {
        let relative = Path::new(path);
        if !relative
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
        {
            return None;
        }
        let bytes = std::fs::read(self.root.join(relative)).ok()?;
        Some(Asset {
            bytes,
            mime_type: mime_type_of(path).into(),
        })
    }
}

/// Enough types for what Vite builds and the sprites.
pub fn mime_type_of(path: &str) -> &'static str {
    let extension = path.rsplit_once('.').map_or("", |(_, extension)| extension);
    match extension.to_ascii_lowercase().as_str() {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json",
        "png" => "image/png",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serves_files_inside_the_folder_only() {
        // Arrange
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("dist");
        std::fs::create_dir_all(root.join("ui")).unwrap();
        std::fs::write(root.join("ui/emotes.png"), b"png").unwrap();
        std::fs::write(parent.path().join("secret.txt"), b"secret").unwrap();
        let assets = DirAssets::new(&root);

        // Act
        let sprite = assets.get("ui/emotes.png");
        let escaped = assets.get("../secret.txt");
        let absolute = assets.get(&parent.path().join("secret.txt").display().to_string());

        // Assert
        assert_eq!(sprite.unwrap().mime_type, "image/png");
        assert_eq!(escaped, None);
        assert_eq!(absolute, None);
    }
}
