# Zom IDE 编辑引擎 RFC（v0.1）

- 状态：Draft
- 作者：Zom Team
- 日期：2026-05-01

## 1. 背景

Zom IDE 进入编辑引擎建设阶段。当前优先目标是先建立“稳定、可扩展、可测试”的编辑内核（Editor Core），为后续语法高亮、LSP、重构、调试等能力提供统一基础。

## 2. 目标（Phase 1）

本阶段仅交付最小可用编辑引擎（MVP）：

1. 支持单文件文本编辑（UTF-8）。
2. 支持插入、删除、替换、粘贴等基础编辑动作。
3. 支持撤销/重做（Undo/Redo）。
4. 支持单光标与多光标（基础行为）。
5. 支持可视区模型（Viewport）和增量渲染信号。
6. 在 10MB 纯文本文件上保持可接受交互性能。

## 3. 非目标（Phase 1 不做）

1. 不实现协作编辑（CRDT/OT）。
2. 不实现重构、调试、代码导航等高级 IDE 能力。
3. 不把语言服务逻辑耦合到编辑内核。
4. 不实现完整插件系统（仅预留事件接口）。

## 4. 核心设计

### 4.1 数据结构

采用 **Rope** 作为文本缓冲核心结构，辅以行索引（Line Index）。

- 选择理由：
1. 对高频插入/删除友好，适合 IDE 交互。
2. 撤销/重做实现自然（基于操作命令）。
3. 对大文件可避免频繁整体拷贝。
4. 便于后续做增量渲染与分段读取，降低大文本拼接成本。

- 实现约束（MVP）：
1. 统一使用 UTF-8 字节 offset 作为内部定位单位。
2. 叶子节点按固定上限分块（如 1-4KB），避免极端碎片化。
3. `LineIndex` 与 `Rope` 同步增量更新；先保证正确性，再做批量优化。

### 4.2 引擎模块

1. `TextBuffer`：维护文本内容与 Rope。
2. `LineIndex`：维护 offset <-> (line, column) 映射。
3. `SelectionModel`：维护光标与选区（含多光标集合）。
4. `CommandStack`：统一封装编辑命令与 Undo/Redo。
5. `ViewportModel`：维护可视窗口状态与脏区更新。
6. `EditorEvents`：对 UI 和语言服务发出增量事件。

### 4.3 命令模型

所有编辑行为统一收敛为命令：

- `insert(offset, text)`
- `delete(range)`
- `replace(range, text)`

Undo/Redo 基于命令逆操作实现，保证行为可回放、可测试。

## 5. 性能与质量目标

### 5.1 性能目标（MVP）

1. 打开 10MB 文本文件：可进入可编辑状态（目标 < 2s，开发机基线）。
2. 连续输入：无明显卡顿（主线程帧掉落可感知阈值以下）。
3. 单次粘贴 10,000 行：可完成并保持可继续编辑。

### 5.2 质量目标

1. 单元测试覆盖 `TextBuffer`、`LineIndex`、`CommandStack` 核心路径。
2. 引入随机编辑序列测试（fuzz/property-based）验证一致性。
3. 对关键状态转移输出调试日志（可开关）。

## 6. 对外接口（草案）

```ts
interface EditorCore {
  apply(command: EditCommand): void;
  undo(): void;
  redo(): void;
  getText(range?: Range): string;
  getSelections(): Selection[];
  setSelections(selections: Selection[]): void;
  getViewport(): ViewportState;
  on(event: EditorEvent, handler: (payload: unknown) => void): () => void;
}
```

## 7. 里程碑（首周）

1. 完成 `TextBuffer(Rope)` 最小实现。
2. 完成 `LineIndex` 与基础定位 API。
3. 完成 `insert/delete/replace + undo/redo`。
4. 完成最小基准测试（打开、输入、粘贴）。
5. 用简易 UI 壳联通输入 -> 引擎 -> 渲染事件链路。

## 8. 风险与缓解

1. **风险**：行索引更新逻辑复杂，易出现边界 bug。  
   **缓解**：先做正确性优先实现，再做增量优化。
2. **风险**：多光标语义不一致。  
   **缓解**：先定义冲突规则（排序、合并、去重）并固化测试。
3. **风险**：性能问题后置发现。  
   **缓解**：首周即引入基准并持续跟踪。

## 9. 待决策问题

1. 文本缓冲是否需要在接口层预留可替换实现（当前默认 Rope）？
2. 增量语法高亮在 Phase 1.5 引入还是 Phase 2 引入？
3. 事件协议是否直接对齐后续 LSP 适配层？
