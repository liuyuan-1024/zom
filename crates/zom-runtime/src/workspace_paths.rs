//! 工作区路径相关能力。

use std::{
    env, fs,
    path::{Path, PathBuf},
};

/// 生成工作区文件的绝对路径。
pub fn workspace_file_absolute_path(workspace_root: &Path, relative_path: &str) -> PathBuf {
    workspace_root.join(relative_path)
}

/// 推断当前进程所在目录，作为工作区根目录的默认值。
pub fn detect_workspace_root() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// 规范化工作区目录：优先返回规范路径，失败时回退到原路径。
pub fn normalize_workspace_root(path: impl Into<PathBuf>) -> PathBuf {
    let path = path.into();
    fs::canonicalize(&path).unwrap_or(path)
}

/// 从工作区根目录推断项目名称。
pub fn project_name_from_root(workspace_root: &Path) -> String {
    workspace_root
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "workspace".into())
}

/// 从相对路径提取文件名，作为标签标题等展示用途。
pub fn file_name_from_path(relative_path: &str) -> String {
    Path::new(relative_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| relative_path.to_string())
}
