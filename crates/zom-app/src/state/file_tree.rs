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
