//! 编辑器草稿存储（自动保存 / 崩溃恢复）。

use std::{
    collections::hash_map::DefaultHasher,
    fs, io,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

const DRAFT_ROOT_DIR: &str = "zom-editor-drafts";
const DRAFT_EXT: &str = "draft";

pub(crate) fn load_draft(workspace_root: &Path, relative_path: &str) -> io::Result<Option<String>> {
    let draft_path = draft_file_path(workspace_root, relative_path);
    match fs::read_to_string(draft_path) {
        Ok(content) => Ok(Some(content)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err),
    }
}

pub(crate) fn store_draft(workspace_root: &Path, relative_path: &str, text: &str) -> io::Result<()> {
    let root = workspace_draft_root(workspace_root);
    fs::create_dir_all(&root)?;
    fs::write(draft_file_path(workspace_root, relative_path), text)
}

pub(crate) fn remove_draft(workspace_root: &Path, relative_path: &str) -> io::Result<()> {
    let draft_path = draft_file_path(workspace_root, relative_path);
    match fs::remove_file(draft_path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

pub(crate) fn draft_file_path(workspace_root: &Path, relative_path: &str) -> PathBuf {
    workspace_draft_root(workspace_root).join(format!("{}.{}", encode_hex(relative_path), DRAFT_EXT))
}

pub(crate) fn workspace_draft_root(workspace_root: &Path) -> PathBuf {
    std::env::temp_dir()
        .join(DRAFT_ROOT_DIR)
        .join(format!("ws-{:016x}", hash_path(workspace_root)))
}

fn hash_path(path: &Path) -> u64 {
    let mut hasher = DefaultHasher::new();
    path.to_string_lossy().hash(&mut hasher);
    hasher.finish()
}

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
    fn draft_path_is_stable_for_same_workspace_and_relative_path() {
        let workspace = std::env::temp_dir().join("zom-draft-store-stable-workspace");
        let relative_path = "a/b/c.txt";
        let first = draft_file_path(&workspace, relative_path);
        let second = draft_file_path(&workspace, relative_path);
        assert_eq!(first, second);
        assert!(first.starts_with(workspace_draft_root(&workspace)));
    }
}
