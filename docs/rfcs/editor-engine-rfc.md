# Zom IDE 编辑引擎 RFC（v0.2）

- 状态：Draft
- 作者：Zom Team
- 日期：2026-05-02

## 1. 背景

Zom 编辑引擎已具备可运行主链路，但“能力在正确层实现”与“可持续演进接口”仍需收口。本文在 v0.1 基础上补齐现状核实、缺口列表与两周执行计划，作为 M0~M5 的落地基线。

## 2. 范围与目标（Phase 1）

1. 支持单文件 UTF-8 文本编辑。
2. 支持插入、删除、替换、查找替换、撤销、重做。
3. 支持单光标稳定行为。
4. 建立 Viewport/Delta 事件协议，避免整文档刷新耦合。
5. 在 1MB~10MB 文档下保持稳定交互性能并可回归。

## 3. 非目标（Phase 1 不做）

1. 协作编辑（CRDT/OT）。
2. 重构、调试、代码导航等高级 IDE 能力。
3. 语言服务逻辑与编辑内核耦合。
4. 完整插件系统。
5. 多光标编辑（`SelectionSet` 接入与跨选区事务语义）。

## 4. 核心设计约束

### 4.1 数据结构

采用 **Rope** 作为文本缓冲核心结构（当前实现：`ropey`）。

实现约束：
1. 内部定位单位统一为 UTF-8 字节 offset。
2. 所有编辑入口都必须通过字符边界校验。
3. 文本归一化策略：编辑内核使用 LF 表示，文件 I/O 侧负责 LF/CRLF 还原。

### 4.2 分层职责（强约束）

1. `zom-protocol`：语义与稳定契约。
2. `zom-text`：文本存储与位置映射。
3. `zom-editor`：编辑行为、事务、历史策略（Undo/Redo）。
4. `zom-runtime`：命令编排与外部副作用（剪贴板、项目/保存、UI action）。
5. `zom-gpui`：渲染与交互绑定。

### 4.3 编辑命令模型

所有文本变更收敛到事务：
1. `insert(offset, text)`
2. `delete(range)`
3. `replace(range, text)`

撤销/重做由 `zom-editor` 的历史栈能力驱动，`zom-runtime` 仅按 buffer 分发调用。

### 4.4 LineIndex 既定方案（Phase 1 定稿）

1. `LineIndex` 在 `zom-text` 作为独立模块实现（与 `TextBuffer` 解耦为“组合关系”）。
2. `TextBuffer` 的位置映射相关 API（`position_to_offset` / `offset_to_position` / `line_count` / `line_len` / `clamp_position`）统一委托 `LineIndex`。
3. `LineIndex` 保持只读语义，内部以 `Rope` 视图计算；Phase 1 不引入额外可变缓存结构。
4. Phase 1 的优先级为“正确性优先 + 可替换接口先稳定”；Phase 1.5 再评估增量缓存优化。

## 5. 现状核实（截至 2026-05-02）

### 5.1 已完成

1. Rope 文本核心与 UTF-8 边界校验已落地。
2. 事务模型（版本校验、变更排序、选区映射）已落地。
3. Undo/Redo 核心能力已下沉到 `zom-editor`，runtime 仅做编排。
4. `SelectAll` 已归位到 `zom-editor` 行为层，不再由 runtime 特判。
5. 现有测试通过：`zom-editor`、`zom-runtime` 编辑链路稳定。

### 5.2 未完成（缺口）

1. `ViewportModel` 与 `EditorEvents` 仍未形成核心协议层，UI 仍偏快照驱动。
2. property/fuzz 随机编辑序列测试尚未接入。

## 6. 两周执行计划（可直接开工）

### Week 1：核心一致性收口

1. 单光标事务与选区语义收口：补齐边界测试矩阵（ASCII/CJK/emoji/组合字符/跨行编辑）。
2. 明确并实现 `LineIndex` 策略（独立模块）。
3. 把 runtime 仍残留的编辑语义特判清零（仅保留副作用编排）。
4. 为 Phase 2 多光标预留协议扩展点（不实现多光标行为）。

门禁：
1. `cargo test -p zom-editor -p zom-runtime` 全绿。
2. 单光标边界用例补齐并无回归。
3. `LineIndex` 验收项通过（见第 7 节补充条款）。

### Week 2：增量渲染协议与性能门禁

1. 定义 `EditorEvents` 最小协议：`Delta`、`SelectionChanged`、`ViewportInvalidated`。
2. 定义 `ViewportState` 与脏区边界（逻辑行范围），并在事件上携带 `dirty_lines`。
3. 协议类型统一迁入 `zom-protocol`（`editor_events` 模块）作为稳定契约，`zom-editor` 仅保留桥接与转换实现。
4. `ViewportInvalidated` 触发源覆盖：滚动、视口尺寸变化、软换行宽度变化（wrap 策略变化）。
5. runtime/gpui 从“整文本快照刷新”迁移到“事件驱动 + 局部重建”。
6. 增加 fuzz/property-based 编辑序列一致性验证。

门禁：
1. `./scripts/bench-editor-core.sh` 稳定产出并通过阈值。
2. 大文本编辑无整屏闪烁，滚动与输入无明显退化。
3. `ViewportInvalidated` 与 `dirty_lines` 在协议层有自动化测试覆盖，且 `zom-editor`/`zom-runtime` 编译回归通过。
4. 至少覆盖 3 类视口触发路径测试：scroll / resize / wrap width change。

## 7. 验收与回滚策略

1. 每项能力都要满足：可演示 + 自动化测试 + 回滚路径。
2. 新增能力必须有降级开关或兼容旧路径。
3. 若跨 3+ crate 修改，必须先做边界复盘再合并。

LineIndex 补充验收项：
1. `zom-text` 中 `TextBuffer` 对外映射行为与改造前保持一致（既有测试全通过）。
2. 新增或更新测试覆盖：ASCII、CJK、emoji、组合字符、CRLF/LF 场景下的位置映射一致性。
3. `cargo test -p zom-text -p zom-editor -p zom-runtime` 全绿。
4. `./scripts/bench-editor-core.sh` 结果不劣于当前基线（至少保持 `PASS`）。

## 8. 风险与缓解

1. 增量事件协议不稳定导致 UI 抖动。  
   缓解：先以最小事件集落地，逐步扩展事件类型。
2. 性能回归后置发现。  
   缓解：每次合并前执行性能基线并留存结果。

## 9. 待决策问题

1. 事件协议是否直接对齐未来 LSP 适配层的数据形态？
2. 多光标能力计划在哪个里程碑进入（建议 Phase 2）？
