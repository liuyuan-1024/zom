//! 本地 UI 资源加载器。
//! 当前主要用于加载工具栏和标题栏使用的 SVG 图标。

use std::{borrow::Cow, fs, path::PathBuf};

use gpui::{AssetSource, Result, SharedString};

/// `zom-gpui` 的本地资源集合。
pub(crate) struct ZomAssets {
    /// 资源根目录。
    base: PathBuf,
}

impl ZomAssets {
    /// 创建默认资源加载器。
    pub(crate) fn new() -> Self {
        Self {
            base: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        }
    }
}

impl AssetSource for ZomAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        fs::read(self.base.join(path))
            .map(|data| Some(Cow::Owned(data)))
            .map_err(Into::into)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        fs::read_dir(self.base.join(path))
            .map(|entries| {
                entries
                    .filter_map(|entry| {
                        entry
                            .ok()
                            .and_then(|entry| entry.file_name().into_string().ok())
                            .map(SharedString::from)
                    })
                    .collect()
            })
            .map_err(Into::into)
    }
}
