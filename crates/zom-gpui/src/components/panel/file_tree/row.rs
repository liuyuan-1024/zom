//! 文件树行渲染与视觉状态规则。

use gpui::{Div, IntoElement, div, prelude::*, px, rgb, svg, transparent_black};
use zom_runtime::state::{FileTreeNode, FileTreeNodeKind};

use super::FILE_TREE_INDENT_STEP;
use crate::icon::AppIcon;
use crate::theme::{color, size};

/// 文件树行组件构建器
pub(super) struct FileTreeRow<'a> {
    /// 当前行绑定的文件树节点。
    node: &'a FileTreeNode,
    /// 当前节点层级，用于计算左缩进。
    depth: usize,
    /// 文件树面板是否拥有焦点（决定是否展示焦点描边）。
    is_panel_focused: bool,
}

impl<'a> FileTreeRow<'a> {
    /// 以节点引用构造行组件。
    pub fn new(node: &'a FileTreeNode) -> Self {
        Self {
            node,
            depth: 0,
            is_panel_focused: false,
        }
    }

    /// 设置节点的层级深度（用于计算左侧缩进）
    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    /// 告知组件当前面板是否获得了逻辑焦点。
    pub fn panel_focused(mut self, focused: bool) -> Self {
        self.is_panel_focused = focused;
        self
    }

    // --- 内部状态判定方法 ---

    /// 判断当前行是否需要展示焦点强调样式。
    fn focus_emphasis_visible(&self) -> bool {
        self.node.is_selected && self.is_panel_focused
    }

    /// 仅在“活动文件”节点上绘制背景高亮，选中态本身不改底色以避免与焦点描边冲突。
    fn row_background_color(&self) -> Option<u32> {
        if self.node.is_active {
            Some(color::COLOR_BG_ACTIVE)
        } else {
            None
        }
    }

    // --- 内部渲染部件方法 ---

    /// 渲染节点名文本，统一承担截断与主文字色策略。
    fn render_label(&self) -> impl IntoElement {
        div()
            .flex_1()
            .text_sm()
            .text_color(rgb(color::COLOR_FG_PRIMARY))
            .overflow_hidden()
            .whitespace_nowrap()
            .child(self.node.name.clone())
    }

    /// 根据节点类型选择文件/目录图标渲染分支。
    fn render_kind_badge(&self) -> impl IntoElement {
        match self.node.kind {
            FileTreeNodeKind::Directory => self.render_folder_icon().into_any_element(),
            FileTreeNodeKind::File => self.render_file_icon().into_any_element(),
        }
    }

    fn render_folder_icon(&self) -> impl IntoElement {
        let icon = file_tree_icon(FileTreeNodeKind::Directory, self.node.is_expanded);

        div()
            .w(px(size::GAP_3))
            .flex()
            .items_center()
            .justify_center()
            .child(
                svg()
                    .path(icon.asset_path())
                    .size(px(size::ICON_MD))
                    .text_color(rgb(color::COLOR_FG_MUTED)),
            )
    }

    /// 渲染文件图标。
    fn render_file_icon(&self) -> impl IntoElement {
        let icon = file_tree_icon(FileTreeNodeKind::File, false);
        div()
            .w(px(size::GAP_3))
            .flex()
            .items_center()
            .justify_center()
            .child(
                svg()
                    .path(icon.asset_path())
                    .size(px(size::ICON_MD))
                    .text_color(rgb(color::COLOR_FG_MUTED)),
            )
    }
}

/// 根据节点类型与展开态选择文件树图标资源。
fn file_tree_icon(kind: FileTreeNodeKind, is_expanded: bool) -> AppIcon {
    match kind {
        FileTreeNodeKind::Directory => {
            if is_expanded {
                AppIcon::FileTreeFolderOpen
            } else {
                AppIcon::FileTreeFolder
            }
        }
        FileTreeNodeKind::File => AppIcon::FileTreeFile,
    }
}

// 核心渲染逻辑：把 FileTreeRow 转为 GPUI 元素并组合视觉状态。
impl<'a> IntoElement for FileTreeRow<'a> {
    /// 为 `Element` 提供语义化类型别名。
    type Element = Div;

    /// 根据选中/激活/焦点状态合成单行样式，保证焦点描边与活动底色可共存。
    fn into_element(self) -> Self::Element {
        let has_focus_emphasis = self.focus_emphasis_visible();

        let mut row = div()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            // 常态保留透明边框，避免焦点态出现时导致行高抖动。
            .border_1()
            .border_color(transparent_black())
            .pl(px(FILE_TREE_INDENT_STEP * self.depth as f32))
            .child(self.render_kind_badge())
            .child(self.render_label());

        if has_focus_emphasis {
            row = row.border_color(rgb(color::COLOR_FG_PRIMARY));
        }

        if let Some(background) = self.row_background_color() {
            row = row.bg(rgb(background));
        }

        row
    }
}

pub(super) fn render(
    node: &FileTreeNode,
    depth: usize,
    is_panel_focused: bool,
) -> impl IntoElement {
    FileTreeRow::new(node)
        .depth(depth)
        .panel_focused(is_panel_focused)
}

// 基于结构体内部规则的单元测试。
#[cfg(test)]
mod tests {
    use super::FileTreeRow;
    use crate::theme::color;
    use zom_runtime::state::{FileTreeNode, FileTreeNodeKind};

    /// 构造测试节点，便于组合“选中/活动”状态并验证视觉规则。
    fn node(is_selected: bool, is_active: bool) -> FileTreeNode {
        FileTreeNode {
            name: "lib.rs".to_string(),
            path: "src/lib.rs".to_string(),
            kind: FileTreeNodeKind::File,
            is_expanded: false,
            is_selected,
            is_active,
            children: Vec::new(),
        }
    }

    #[test]
    /// 仅选中不激活时，不应出现活动底色。
    fn selected_node_uses_no_background_even_when_panel_has_focus() {
        let selected = node(true, false);
        assert_eq!(FileTreeRow::new(&selected).row_background_color(), None);
    }

    #[test]
    /// 活动节点即使面板失焦也应保留活动底色。
    fn active_node_stays_highlighted_without_panel_focus() {
        let active = node(true, true);
        assert_eq!(
            FileTreeRow::new(&active).row_background_color(),
            Some(color::COLOR_BG_ACTIVE)
        );
    }

    #[test]
    /// 焦点描边只由“选中 + 面板聚焦”共同决定。
    fn focus_emphasis_tracks_selected_focus_state_even_for_active_node() {
        let active_selected = node(true, true);

        assert!(
            FileTreeRow::new(&active_selected)
                .panel_focused(true)
                .focus_emphasis_visible()
        );
        assert!(
            !FileTreeRow::new(&active_selected)
                .panel_focused(false)
                .focus_emphasis_visible()
        );
    }
}
