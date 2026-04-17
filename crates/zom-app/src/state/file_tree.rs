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

#[cfg(test)]
mod tests {
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
}
