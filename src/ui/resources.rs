use std::collections::HashMap;
use std::path::{Path, PathBuf};

use gdk_pixbuf::Pixbuf;

/// Represents a loaded image – either a raster Pixbuf or SVG tree.
pub enum GameImage {
    Raster(Pixbuf),
    Svg { tree: resvg::usvg::Tree },
}

impl GameImage {
    /// Get the native width of the image.
    #[allow(dead_code)]
    pub fn width(&self) -> f64 {
        match self {
            GameImage::Raster(pb) => pb.width() as f64,
            GameImage::Svg { tree } => tree.size().width() as f64,
        }
    }

    /// Get the native height of the image.
    #[allow(dead_code)]
    pub fn height(&self) -> f64 {
        match self {
            GameImage::Raster(pb) => pb.height() as f64,
            GameImage::Svg { tree } => tree.size().height() as f64,
        }
    }
}

/// All game images loaded from the resources directory.
pub struct GameResources {
    images: HashMap<String, GameImage>,
    #[allow(dead_code)]
    pub res_dir: PathBuf,
}

impl GameResources {
    /// Load all needed images from the given directory.
    /// Automatically picks .svg if available, otherwise .png.
    pub fn load<P: AsRef<Path>>(dir: P) -> Self {
        let dir = dir.as_ref().to_path_buf();
        let mut images = HashMap::new();

        let files = [
            "background",
            "grid",
            "banana",
            "1b",
            "2b",
            "3b",
            "4b",
            "1s",
            "2s",
            "3s",
            "4s",
            "horizontal",
            "vertical",
            "row1",
            "row2",
            "row_pre_last",
            "row_last",
            "won",
            "lost",
            "drawn",
            "selected",
            "tip",
            "shadow",
            "flag_blue",
            "flag_red",
            "icon",
        ];

        for name in &files {
            // Prefer SVG if it exists
            let svg_path = dir.join(format!("{}.svg", name));
            let png_path = dir.join(format!("{}.png", name));

            if svg_path.exists() {
                match Self::load_svg(&svg_path) {
                    Some(img) => {
                        images.insert(name.to_string(), img);
                        continue;
                    }
                    None => {
                        eprintln!("Warning: could not load SVG {}", svg_path.display());
                    }
                }
            }

            match Pixbuf::from_file(&png_path) {
                Ok(pb) => {
                    images.insert(name.to_string(), GameImage::Raster(pb));
                }
                Err(e) => {
                    eprintln!("Warning: could not load {}: {}", png_path.display(), e);
                }
            }
        }

        Self {
            images,
            res_dir: dir,
        }
    }

    fn load_svg(path: &Path) -> Option<GameImage> {
        let data = std::fs::read(path).ok()?;
        let opt = resvg::usvg::Options::default();
        let tree = resvg::usvg::Tree::from_data(&data, &opt).ok()?;
        Some(GameImage::Svg { tree })
    }

    /// Get an image by name (without extension).
    pub fn get(&self, name: &str) -> Option<&GameImage> {
        self.images.get(name)
    }

    /// Get bomb texture by value (0–3).
    pub fn bomb(&self, value: i32) -> Option<&GameImage> {
        let name = format!("{}b", value + 1);
        self.images.get(&name)
    }

    /// Get stone texture by value (0–3).
    pub fn stone(&self, value: i32) -> Option<&GameImage> {
        let name = format!("{}s", value + 1);
        self.images.get(&name)
    }

    /// Get tower row texture by index.
    pub fn tower_row(&self, idx: usize) -> Option<&GameImage> {
        let names = ["row1", "row2", "row_pre_last", "row_last"];
        names.get(idx).and_then(|n| self.images.get(*n))
    }

    /// Get win/loss/draw overlay (0=won, 1=lost, 2=drawn).
    pub fn outcome_overlay(&self, idx: usize) -> Option<&GameImage> {
        let names = ["won", "lost", "drawn"];
        names.get(idx).and_then(|n| self.images.get(*n))
    }
}
