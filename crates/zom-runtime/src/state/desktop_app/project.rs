//! 项目切换、文件打开与保存逻辑

use std::fs;
use std::path::PathBuf;

use zom_editor::EditorState;
use zom_protocol::BufferId;
use zom_text_tokens::{LF_CHAR, LineEnding};

use crate::{
    buffer_preview, draft_store,
    state::{FileTreeNodeKind, FileTreeState, TabState},
    workspace_paths,
};

use super::{
    DesktopAppState, DesktopNotificationEvent, DesktopNotificationLevel, DesktopNotificationSource,
};

impl DesktopAppState {
    /// 切换当前工作区项目，并重建文件树。
    pub fn switch_project(&mut self, project_root: impl Into<PathBuf>) {
        let project_root = workspace_paths::normalize_workspace_root(project_root.into());
        self.project_name = workspace_paths::project_name_from_root(&project_root);
        self.project_root = project_root.clone();
        self.file_tree = FileTreeState::from_workspace_root(&project_root);

        // 旧项目打开的标签页路径不再可信，切换项目时统一清空。
        self.pane.tabs.clear();
        self.pane.active_tab_index = None;
        self.clear_editor_states();
        self.sync_tool_bar_with_active_tab();
    }

    /// 处理文件树节点激活，并同步工作区状态。
    pub fn activate_file_tree_node(&mut self, relative_path: &str, kind: FileTreeNodeKind) {
        match kind {
            FileTreeNodeKind::Directory => self.file_tree.toggle_directory(relative_path),
            FileTreeNodeKind::File => {
                self.file_tree.activate_file(relative_path);
                if self.open_file_in_pane(relative_path) {
                    self.focus_editor();
                }
            }
        }
    }

    /// 在当前 Pane 打开文件：已打开则切换并刷新内容，未打开则新增标签页。
    ///
    /// 对已打开标签会复用原 `buffer_id`，以保持外围引用稳定。
    pub(super) fn open_file_in_pane(&mut self, relative_path: &str) -> bool {
        let absolute_path =
            workspace_paths::workspace_file_absolute_path(&self.project_root, relative_path);
        let Ok(buffer_preview::BufferPreview {
            mut editor_state,
            line_ending,
            ..
        }) = buffer_preview::load_buffer_preview(&absolute_path)
        else {
            return false;
        };
        self.restore_editor_draft_if_exists(relative_path, &mut editor_state);
        let language = workspace_paths::language_from_path(relative_path);

        if let Some(tab_index) = self
            .pane
            .tabs
            .iter()
            .position(|tab| tab.relative_path == relative_path)
        {
            if let Some(existing_tab) = self.pane.tabs.get_mut(tab_index) {
                let buffer_id = existing_tab.buffer_id;
                existing_tab.language = language;
                existing_tab.line_ending = line_ending;
                self.replace_editor_state(buffer_id, editor_state);
                self.editor_histories.remove(&buffer_id);
            }
            self.pane.active_tab_index = Some(tab_index);
            self.sync_file_tree_with_active_tab();
            self.sync_tool_bar_with_active_tab();
            return true;
        }

        let next_buffer_id = self
            .pane
            .tabs
            .iter()
            .map(|tab| tab.buffer_id.value())
            .max()
            .unwrap_or(0)
            + 1;

        let buffer_id = BufferId::new(next_buffer_id);
        self.replace_editor_state(buffer_id, editor_state);
        self.pane.tabs.push(TabState {
            buffer_id,
            title: workspace_paths::file_name_from_path(relative_path),
            relative_path: relative_path.to_string(),
            language,
            line_ending,
        });
        self.pane.active_tab_index = Some(self.pane.tabs.len() - 1);
        self.sync_file_tree_with_active_tab();
        self.sync_tool_bar_with_active_tab();
        true
    }

    /// 保存当前活动标签页到磁盘。
    ///
    /// 保存成功后会清理对应草稿；草稿清理失败不会影响“文件已保存”这一主流程结果。
    pub(super) fn save_active_editor_buffer(&mut self) {
        let Some(active_tab) = self.pane.active_tab().cloned() else {
            return;
        };
        let Some(editor_state) = self.editor_state(active_tab.buffer_id).cloned() else {
            return;
        };

        let absolute_path = workspace_paths::workspace_file_absolute_path(
            &self.project_root,
            &active_tab.relative_path,
        );
        let content = text_with_line_ending(&editor_state.text(), active_tab.line_ending());
        let event = match fs::write(&absolute_path, content) {
            Ok(_) => {
                if let Err(error) =
                    draft_store::remove_draft(&self.project_root, &active_tab.relative_path)
                {
                    self.publish_notification_event(
                        DesktopNotificationEvent::new(
                            DesktopNotificationLevel::Warning,
                            DesktopNotificationSource::System,
                            format!("草稿清理失败 {} ({error})", active_tab.relative_path),
                        )
                        .with_dedupe_key(format!(
                            "workspace:draft:clear:error:{}",
                            active_tab.relative_path
                        )),
                    );
                }
                DesktopNotificationEvent::new(
                    DesktopNotificationLevel::Info,
                    DesktopNotificationSource::Workspace,
                    format!("已保存 {}", active_tab.relative_path),
                )
                .is_user_initiated()
                .with_dedupe_key(format!("workspace:save:{}", active_tab.relative_path))
            }
            Err(error) => DesktopNotificationEvent::new(
                DesktopNotificationLevel::Error,
                DesktopNotificationSource::Workspace,
                format!("保存失败 {} ({error})", active_tab.relative_path),
            )
            .is_user_initiated()
            .with_dedupe_key(format!("workspace:save:error:{}", active_tab.relative_path)),
        };
        self.publish_notification_event(event);
    }

    /// 将指定缓冲区的当前文本持久化为草稿文件。
    ///
    /// 草稿是“恢复兜底”，不是正式保存；写入失败会告警但不阻断编辑流程。
    pub(super) fn persist_editor_draft(&mut self, buffer_id: BufferId, state: &EditorState) {
        let Some(relative_path) = self
            .pane
            .tabs
            .iter()
            .find(|tab| tab.buffer_id == buffer_id)
            .map(|tab| tab.relative_path.clone())
        else {
            return;
        };

        if let Err(error) =
            draft_store::store_draft(&self.project_root, &relative_path, &state.text())
        {
            self.publish_notification_event(
                DesktopNotificationEvent::new(
                    DesktopNotificationLevel::Warning,
                    DesktopNotificationSource::System,
                    format!("草稿自动保存失败 {} ({error})", relative_path),
                )
                .with_dedupe_key(format!("workspace:draft:write:error:{}", relative_path)),
            );
        }
    }

    /// 若存在未保存草稿且内容与当前文本不同，则恢复草稿并通知用户。
    ///
    /// 通过文本对比避免“文件已保存但残留旧草稿”时误覆盖磁盘最新内容。
    fn restore_editor_draft_if_exists(
        &mut self,
        relative_path: &str,
        editor_state: &mut EditorState,
    ) {
        match draft_store::load_draft(&self.project_root, relative_path) {
            Ok(Some(draft_text)) if draft_text != editor_state.text() => {
                *editor_state = EditorState::from_text(draft_text);
                self.publish_notification_event(
                    DesktopNotificationEvent::new(
                        DesktopNotificationLevel::Info,
                        DesktopNotificationSource::Workspace,
                        format!("已恢复未保存草稿 {}", relative_path),
                    )
                    .with_dedupe_key(format!("workspace:draft:restore:{}", relative_path)),
                );
            }
            Ok(_) => {}
            Err(error) => {
                self.publish_notification_event(
                    DesktopNotificationEvent::new(
                        DesktopNotificationLevel::Warning,
                        DesktopNotificationSource::System,
                        format!("草稿读取失败 {} ({error})", relative_path),
                    )
                    .with_dedupe_key(format!("workspace:draft:read:error:{}", relative_path)),
                );
            }
        }
    }
}

/// 按标签页记录的换行风格把内存文本编码为落盘文本。
///
/// 编辑器内部统一使用 LF，保存时再按文件原风格回写以减少无关 diff。
fn text_with_line_ending(text: &str, line_ending: LineEnding) -> String {
    if matches!(line_ending, LineEnding::Lf) {
        return text.to_string();
    }
    text.replace(LF_CHAR, line_ending.as_str())
}
