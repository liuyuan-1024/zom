//! 编辑器草稿存储（自动保存 / 崩溃恢复）。

use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    io,
    path::{Path, PathBuf},
};

const DRAFT_ROOT_DIR: &str = "zom-editor-drafts";
const DRAFT_EXT: &str = "draft";

/// 读取指定文件的草稿内容。
///
/// `NotFound` 被视为“无草稿”而非错误，便于调用方直接做恢复分支。
pub(crate) fn load_draft(workspace_root: &Path, relative_path: &str) -> io::Result<Option<String>> {
    let draft_path = draft_file_path(workspace_root, relative_path);
    match fs::read_to_string(draft_path) {
        Ok(content) => Ok(Some(content)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err),
    }
}

/// 持久化草稿文本到临时目录。
///
/// 会先确保工作区草稿目录存在；覆盖写入是预期行为。
pub(crate) fn store_draft(
    workspace_root: &Path,
    relative_path: &str,
    text: &str,
) -> io::Result<()> {
    let root = workspace_draft_root(workspace_root);
    fs::create_dir_all(&root)?;
    fs::write(draft_file_path(workspace_root, relative_path), text)
}

/// 删除草稿文件。
///
/// 草稿不存在时静默成功，避免把清理动作升级为用户可见错误。
pub(crate) fn remove_draft(workspace_root: &Path, relative_path: &str) -> io::Result<()> {
    let draft_path = draft_file_path(workspace_root, relative_path);
    match fs::remove_file(draft_path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

/// 计算某个工作区文件对应的草稿文件绝对路径。
///
/// 文件名使用十六进制编码后的相对路径，避免目录穿透和非法字符问题。
pub(crate) fn draft_file_path(workspace_root: &Path, relative_path: &str) -> PathBuf {
    workspace_draft_root(workspace_root).join(format!(
        "{}.{}",
        encode_hex(relative_path),
        DRAFT_EXT
    ))
}

/// 计算工作区对应的草稿根目录。
///
/// 目录名基于工作区路径哈希，目的是在同一台机器上隔离不同项目草稿空间。
pub(crate) fn workspace_draft_root(workspace_root: &Path) -> PathBuf {
    std::env::temp_dir()
        .join(DRAFT_ROOT_DIR)
        .join(format!("ws-{:016x}", hash_path(workspace_root)))
}

/// 对工作区路径做稳定哈希。
///
/// 该哈希仅用于命名隔离，不用于安全场景。
fn hash_path(path: &Path) -> u64 {
    let mut hasher = DefaultHasher::new();
    path.to_string_lossy().hash(&mut hasher);
    hasher.finish()
}

/// 把相对路径编码为十六进制文件名，避免路径分隔符和非法字符影响临时目录落盘。
fn encode_hex(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let bytes = value.as_bytes();
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{draft_file_path, load_draft, remove_draft, store_draft, workspace_draft_root};

    #[test]
    /// 计算草稿结果。
    fn store_load_and_remove_draft_roundtrip() {
        let workspace = std::env::temp_dir().join("zom-draft-store-test-workspace");
        let relative_path = "src/main.rs";
        let text = "fn main() {\n    println!(\"hi\");\n}\n";

        store_draft(&workspace, relative_path, text).expect("store draft");
        let loaded = load_draft(&workspace, relative_path).expect("load draft");
        assert_eq!(loaded.as_deref(), Some(text));

        remove_draft(&workspace, relative_path).expect("remove draft");
        let after_remove = load_draft(&workspace, relative_path).expect("load removed draft");
        assert_eq!(after_remove, None);
    }

    #[test]
    /// 计算路径工作区结果。
    fn draft_path_is_stable_for_same_workspace_and_relative_path() {
        let workspace = std::env::temp_dir().join("zom-draft-store-stable-workspace");
        let relative_path = "a/b/c.txt";
        let first = draft_file_path(&workspace, relative_path);
        let second = draft_file_path(&workspace, relative_path);
        assert_eq!(first, second);
        assert!(first.starts_with(workspace_draft_root(&workspace)));
    }
}
