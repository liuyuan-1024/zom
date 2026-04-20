//! 文件树状态与导航行为模型。

use std::{
    fs,
    path::{Path, PathBuf},
};

/// 文件树节点类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileTreeNodeKind {
    /// 目录节点。
    Directory,
    /// 文件节点。
    File,
}

/// 文件树中的单个节点。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTreeNode {
    /// 节点显示名称。
    pub name: String,
    /// 节点在工作区中的相对路径。
    pub path: String,
    /// 节点类型。
    pub kind: FileTreeNodeKind,
    /// 当前节点是否处于展开状态。
    pub is_expanded: bool,
    /// 当前节点是否被文件树选中。
    pub is_selected: bool,
    /// 当前节点是否对应激活文件。
    pub is_active: bool,
    /// 子节点列表。
    pub children: Vec<FileTreeNode>,
}

/// 文件树展示信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTreeState {
    /// 面板标题。
    pub title: String,
    /// 根节点列表。
    pub roots: Vec<FileTreeNode>,
}

impl FileTreeState {
    /// 从真实工作区目录构建文件树状态。
    pub fn from_workspace_root(workspace_root: &Path) -> Self {
        let root_name = workspace_root
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| workspace_root.display().to_string());

        Self {
            title: "EXPLORER".into(),
            roots: vec![directory_node(
                root_name,
                String::new(),
                true,
                collect_directory_children(workspace_root, Path::new("")),
            )],
        }
    }

    /// 折叠或展开指定目录节点。
    pub fn toggle_directory(&mut self, relative_path: &str) {
        for root in &mut self.roots {
            if toggle_directory_node(root, relative_path) {
                break;
            }
        }
    }

    /// 将指定文件标记为当前激活项，并清理其他节点的激活态。
    pub fn activate_file(&mut self, relative_path: &str) {
        for root in &mut self.roots {
            activate_file_node(root, relative_path);
        }
    }

    /// 选中下一条可见节点。
    pub fn select_next_visible(&mut self) {
        self.move_selection(1);
    }

    /// 选中上一条可见节点。
    pub fn select_prev_visible(&mut self) {
        self.move_selection(-1);
    }

    /// 展开选中的目录，或在已展开目录中下探到第一个子节点。
    pub fn expand_or_descend_selected(&mut self) {
        let Some(selected) = self.selected_visible_node_or_first() else {
            return;
        };

        if matches!(selected.kind, FileTreeNodeKind::Directory) {
            if !selected.is_expanded {
                self.toggle_directory(&selected.path);
            } else if let Some(first_child_path) = selected.first_child_path {
                self.select_only(&first_child_path);
            }
        }
    }

    /// 折叠选中的目录，或回到父节点。
    pub fn collapse_or_ascend_selected(&mut self) {
        let Some(selected) = self.selected_visible_node_or_first() else {
            return;
        };

        if matches!(selected.kind, FileTreeNodeKind::Directory) && selected.is_expanded {
            self.toggle_directory(&selected.path);
            return;
        }

        if let Some(parent_path) = selected.parent_path {
            self.select_only(&parent_path);
        }
    }

    /// 返回当前选中节点的路径和类型。
    pub fn selected_node(&self) -> Option<(String, FileTreeNodeKind)> {
        let visible_nodes = self.visible_nodes();
        let selected_path = self.selected_path()?;
        visible_nodes
            .into_iter()
            .find(|node| node.path == selected_path)
            .map(|node| (node.path, node.kind))
    }

    /// 确保当前至少有一条可见选中节点。
    /// 若无选中或选中项不可见，则自动选中第一条可见节点。
    /// 返回值表示是否发生了状态变更。
    pub fn ensure_selection(&mut self) -> bool {
        let visible_nodes = self.visible_nodes();
        let Some(first_visible_path) = visible_nodes.first().map(|node| node.path.clone()) else {
            return false;
        };

        if let Some(selected_path) = self.selected_path()
            && visible_nodes.iter().any(|node| node.path == selected_path)
        {
            return false;
        }

        self.select_only(&first_visible_path);
        true
    }

    fn move_selection(&mut self, direction: isize) {
        let visible_nodes = self.visible_nodes();
        if visible_nodes.is_empty() {
            return;
        }

        let selected_path = self.selected_path();
        let target_index = match selected_path
            .as_ref()
            .and_then(|path| visible_nodes.iter().position(|node| &node.path == path))
        {
            Some(current_index) => {
                if direction > 0 {
                    (current_index + 1).min(visible_nodes.len() - 1)
                } else {
                    current_index.saturating_sub(1)
                }
            }
            None => {
                if direction > 0 {
                    0
                } else {
                    visible_nodes.len() - 1
                }
            }
        };

        self.select_only(&visible_nodes[target_index].path);
    }

    fn selected_visible_node_or_first(&mut self) -> Option<VisibleNode> {
        let visible_nodes = self.visible_nodes();
        if visible_nodes.is_empty() {
            return None;
        }

        if let Some(selected_path) = self.selected_path()
            && let Some(node) = visible_nodes
                .iter()
                .find(|candidate| candidate.path == selected_path)
        {
            return Some(node.clone());
        }

        let first = visible_nodes[0].clone();
        self.select_only(&first.path);
        Some(first)
    }

    fn select_only(&mut self, relative_path: &str) {
        for root in &mut self.roots {
            select_only_node(root, relative_path);
        }
    }

    fn selected_path(&self) -> Option<String> {
        self.roots
            .iter()
            .find_map(find_selected_node_path)
            .map(ToString::to_string)
    }

    fn visible_nodes(&self) -> Vec<VisibleNode> {
        let mut visible_nodes = Vec::new();
        for root in &self.roots {
            collect_visible_nodes(root, None, &mut visible_nodes);
        }
        visible_nodes
    }
}

/// 递归切换目录节点的展开态。返回值表示是否命中目标节点。
fn toggle_directory_node(node: &mut FileTreeNode, relative_path: &str) -> bool {
    if node.path == relative_path && matches!(node.kind, FileTreeNodeKind::Directory) {
        node.is_expanded = !node.is_expanded;
        return true;
    }

    for child in &mut node.children {
        if toggle_directory_node(child, relative_path) {
            return true;
        }
    }

    false
}

/// 递归激活文件节点。返回值表示该子树中是否命中目标文件。
fn activate_file_node(node: &mut FileTreeNode, relative_path: &str) -> bool {
    let is_target_file = node.path == relative_path && matches!(node.kind, FileTreeNodeKind::File);
    node.is_active = is_target_file;
    node.is_selected = is_target_file;

    let mut contains_target = is_target_file;
    for child in &mut node.children {
        if activate_file_node(child, relative_path) {
            contains_target = true;
        }
    }

    if matches!(node.kind, FileTreeNodeKind::Directory) && contains_target {
        node.is_expanded = true;
    }

    contains_target
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VisibleNode {
    path: String,
    kind: FileTreeNodeKind,
    parent_path: Option<String>,
    is_expanded: bool,
    first_child_path: Option<String>,
}

fn collect_visible_nodes(
    node: &FileTreeNode,
    parent_path: Option<&str>,
    visible_nodes: &mut Vec<VisibleNode>,
) {
    visible_nodes.push(VisibleNode {
        path: node.path.clone(),
        kind: node.kind,
        parent_path: parent_path.map(ToString::to_string),
        is_expanded: node.is_expanded,
        first_child_path: node.children.first().map(|child| child.path.clone()),
    });

    if matches!(node.kind, FileTreeNodeKind::Directory) && node.is_expanded {
        for child in &node.children {
            collect_visible_nodes(child, Some(&node.path), visible_nodes);
        }
    }
}

fn select_only_node(node: &mut FileTreeNode, relative_path: &str) {
    node.is_selected = node.path == relative_path;
    for child in &mut node.children {
        select_only_node(child, relative_path);
    }
}

fn find_selected_node_path(node: &FileTreeNode) -> Option<&str> {
    if node.is_selected {
        return Some(&node.path);
    }

    node.children.iter().find_map(find_selected_node_path)
}

const SKIPPED_DIRECTORY_NAMES: &[&str] = &[".git", "node_modules", "target"];
const SKIPPED_FILE_NAMES: &[&str] = &[".DS_Store"];

fn collect_directory_children(absolute_dir: &Path, relative_dir: &Path) -> Vec<FileTreeNode> {
    let Ok(entries) = fs::read_dir(absolute_dir) else {
        return Vec::new();
    };

    let mut directories = Vec::new();
    let mut files = Vec::new();

    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        if file_type.is_dir() && SKIPPED_DIRECTORY_NAMES.contains(&name.as_str()) {
            continue;
        }
        if file_type.is_file() && SKIPPED_FILE_NAMES.contains(&name.as_str()) {
            continue;
        }

        let relative_path = path_join(relative_dir, &name);
        let relative_path_string = to_unix_style_path(&relative_path);

        if file_type.is_dir() {
            let children = collect_directory_children(&entry.path(), &relative_path);
            directories.push(directory_node(name, relative_path_string, false, children));
            continue;
        }

        if file_type.is_file() {
            files.push(file_node(name, relative_path_string));
        }
    }

    sort_nodes_by_name(&mut directories);
    sort_nodes_by_name(&mut files);
    directories.extend(files);
    directories
}

fn sort_nodes_by_name(nodes: &mut [FileTreeNode]) {
    nodes.sort_by_cached_key(|node| node.name.to_lowercase());
}

fn path_join(base: &Path, child_name: &str) -> PathBuf {
    if base.as_os_str().is_empty() {
        PathBuf::from(child_name)
    } else {
        base.join(child_name)
    }
}

fn to_unix_style_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn directory_node(
    name: impl Into<String>,
    path: impl Into<String>,
    is_expanded: bool,
    children: Vec<FileTreeNode>,
) -> FileTreeNode {
    FileTreeNode {
        name: name.into(),
        path: path.into(),
        kind: FileTreeNodeKind::Directory,
        is_expanded,
        is_selected: false,
        is_active: false,
        children,
    }
}

fn file_node(name: impl Into<String>, path: impl Into<String>) -> FileTreeNode {
    FileTreeNode {
        name: name.into(),
        path: path.into(),
        kind: FileTreeNodeKind::File,
        is_expanded: false,
        is_selected: false,
        is_active: false,
        children: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{FileTreeNode, FileTreeNodeKind, FileTreeState};

    #[test]
    fn toggle_directory_updates_expanded_flag() {
        let mut tree = FileTreeState {
            title: "EXPLORER".into(),
            roots: vec![directory("root", "root", true, vec![])],
        };

        tree.toggle_directory("root");
        assert!(!tree.roots[0].is_expanded);

        tree.toggle_directory("root");
        assert!(tree.roots[0].is_expanded);
    }

    #[test]
    fn activate_file_marks_only_target_node() {
        let mut tree = FileTreeState {
            title: "EXPLORER".into(),
            roots: vec![directory(
                "root",
                "root",
                false,
                vec![
                    file("a.rs", "root/a.rs"),
                    directory(
                        "src",
                        "root/src",
                        false,
                        vec![file("b.rs", "root/src/b.rs")],
                    ),
                ],
            )],
        };

        tree.activate_file("root/src/b.rs");

        let root = &tree.roots[0];
        assert!(root.is_expanded);
        assert!(!root.children[0].is_active);
        assert!(root.children[1].is_expanded);
        assert!(root.children[1].children[0].is_active);
        assert!(root.children[1].children[0].is_selected);
    }

    #[test]
    fn keyboard_navigation_moves_selection_between_visible_nodes() {
        let mut tree = FileTreeState {
            title: "EXPLORER".into(),
            roots: vec![directory(
                "root",
                "root",
                true,
                vec![file("a.rs", "root/a.rs"), file("b.rs", "root/b.rs")],
            )],
        };

        tree.select_next_visible();
        assert_eq!(
            tree.selected_node(),
            Some(("root".to_string(), FileTreeNodeKind::Directory))
        );

        tree.select_next_visible();
        assert_eq!(
            tree.selected_node(),
            Some(("root/a.rs".to_string(), FileTreeNodeKind::File))
        );

        tree.select_prev_visible();
        assert_eq!(
            tree.selected_node(),
            Some(("root".to_string(), FileTreeNodeKind::Directory))
        );
    }

    #[test]
    fn keyboard_expand_and_collapse_operates_on_selected_directory() {
        let mut tree = FileTreeState {
            title: "EXPLORER".into(),
            roots: vec![directory(
                "root",
                "root",
                false,
                vec![file("a.rs", "root/a.rs"), file("b.rs", "root/b.rs")],
            )],
        };

        tree.select_next_visible();
        tree.expand_or_descend_selected();
        assert!(tree.roots[0].is_expanded);

        tree.collapse_or_ascend_selected();
        assert!(!tree.roots[0].is_expanded);
    }

    #[test]
    fn ensure_selection_selects_first_visible_node_when_none_selected() {
        let mut tree = FileTreeState {
            title: "EXPLORER".into(),
            roots: vec![directory(
                "root",
                "root",
                true,
                vec![file("a.rs", "root/a.rs"), file("b.rs", "root/b.rs")],
            )],
        };

        assert!(tree.ensure_selection());
        assert_eq!(
            tree.selected_node(),
            Some(("root".to_string(), FileTreeNodeKind::Directory))
        );
    }

    #[test]
    fn ensure_selection_reselects_when_current_selection_is_not_visible() {
        let mut tree = FileTreeState {
            title: "EXPLORER".into(),
            roots: vec![directory(
                "root",
                "root",
                true,
                vec![directory(
                    "src",
                    "root/src",
                    false,
                    vec![file("main.rs", "root/src/main.rs")],
                )],
            )],
        };
        tree.roots[0].children[0].children[0].is_selected = true;

        assert!(tree.ensure_selection());
        assert_eq!(
            tree.selected_node(),
            Some(("root".to_string(), FileTreeNodeKind::Directory))
        );
    }

    #[test]
    fn from_workspace_root_loads_real_directory_entries() {
        let workspace = create_temp_workspace("load-real-directory");

        fs::create_dir_all(workspace.join("src")).expect("create src directory");
        fs::write(workspace.join("Cargo.toml"), "[package]").expect("create Cargo.toml");
        fs::write(workspace.join("src/main.rs"), "fn main() {}").expect("create main.rs");

        let tree = FileTreeState::from_workspace_root(&workspace);

        assert_eq!(tree.roots.len(), 1);
        assert_eq!(tree.roots[0].name, workspace_file_name(&workspace));
        assert_eq!(tree.roots[0].path, "");
        assert!(tree.roots[0].is_expanded);
        assert_eq!(
            tree.roots[0]
                .children
                .iter()
                .map(|node| node.path.as_str())
                .collect::<Vec<_>>(),
            vec!["src", "Cargo.toml"]
        );
        assert_eq!(
            tree.roots[0].children[0]
                .children
                .iter()
                .map(|node| node.path.as_str())
                .collect::<Vec<_>>(),
            vec!["src/main.rs"]
        );

        remove_temp_workspace(workspace);
    }

    #[test]
    fn from_workspace_root_skips_common_generated_directories() {
        let workspace = create_temp_workspace("skip-generated-directories");

        fs::create_dir_all(workspace.join("target/debug")).expect("create target directory");
        fs::write(workspace.join("README.md"), "hello").expect("create README.md");

        let tree = FileTreeState::from_workspace_root(&workspace);
        let child_paths = tree.roots[0]
            .children
            .iter()
            .map(|node| node.path.as_str())
            .collect::<Vec<_>>();

        assert_eq!(child_paths, vec!["README.md"]);

        remove_temp_workspace(workspace);
    }

    fn directory(
        name: &str,
        path: &str,
        is_expanded: bool,
        children: Vec<FileTreeNode>,
    ) -> FileTreeNode {
        FileTreeNode {
            name: name.into(),
            path: path.into(),
            kind: FileTreeNodeKind::Directory,
            is_expanded,
            is_selected: false,
            is_active: false,
            children,
        }
    }

    fn file(name: &str, path: &str) -> FileTreeNode {
        FileTreeNode {
            name: name.into(),
            path: path.into(),
            kind: FileTreeNodeKind::File,
            is_expanded: false,
            is_selected: false,
            is_active: false,
            children: Vec::new(),
        }
    }

    fn create_temp_workspace(name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("current time should be after unix epoch")
            .as_nanos();
        let workspace = std::env::temp_dir().join(format!("zom-file-tree-{name}-{timestamp}"));
        fs::create_dir_all(&workspace).expect("create temp workspace directory");
        workspace
    }

    fn remove_temp_workspace(path: PathBuf) {
        fs::remove_dir_all(path).expect("remove temp workspace");
    }

    fn workspace_file_name(path: &PathBuf) -> String {
        path.file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string())
    }
}
