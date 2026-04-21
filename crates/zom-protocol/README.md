# zom-protocol 是 zom 的“公共语言层”

> 跨 editor、workspace、input、gpui 都成立的稳定抽象。

可以把它理解成：
- zom 的词汇表
- zom 的协议层
- zom 的最小共享模型

## 边界执行纪律（强制）
1. `zom-protocol` 只承载稳定协议与共享类型，禁止承载上层业务编排与 UI 细节。
2. 其他 crate 禁止重复定义 `zom-protocol` 已存在的稳定类型语义（含 1:1 镜像结构）。
3. 开发前先判定职责归属；若出现边界争议，按 `docs/开发规范手册` 与 `docs/Crate边界契约.md` 先对齐后开发。

## zom-protocol 应该放些什么？
1. 基础值对象
这些是所有层都可能用到的稳定类型：
Position
Range
Selection
SelectionSet
Offset
Extent / Size
Direction
Axis
这些类型的特点是：语义清晰、不依赖具体实现、在 editor 和 UI 之间都成立。

2. 命令模型
CommandKind、CommandInvocation、EditorInvocation/EditorAction、WorkspaceAction。因为这些是全系统共享协议。
另外，`command::kind` 负责维护命令目录（Command Catalog），统一提供：
- 单一声明源（`CommandKindSpec`）
- 命令稳定键（`CommandKind`）与稳定字符串标识（`CommandKindId`）
- 命令元信息（`CommandMeta`）
- 默认快捷键元数据（`default_shortcut_bindings`）

3. 输入协议与默认解析模型
`zom-protocol::input` 既定义输入协议，也承载默认 keymap 解析实现。
适合放：
Keystroke
Modifiers
KeyCode
InputContext
FocusTarget
InputResult / InputResolution
因为：
zom-protocol::input 自身要生产/消费它
zom-gpui 要桥接它
zom-workspace 要提供 context facts
zom-editor 可能要读取一部分上下文

4. 标识与元信息类型
例如：
BufferId
PaneId
TabId
WorkspaceId
CommandKind
这些类型最好做成强类型，不要满地 u64 和 String。

5. 通用结果与能力描述
例如：
CanExecute
CommandStatus
EnabledState
ReadOnly
Dirty
Version
这类抽象如果足够通用，可以放 core。
但要很克制，不要把业务状态全塞进来。

## zom-protocol 的层级划分
zom-protocol
├── geometry      // 位置、范围、方向
├── selection     // 选区模型
├── command       // 命令语义
├── input         // 输入协议 + 默认解析
├── ids           // 强类型 ID
└── state         // 少量通用状态类型

# 命令语义（CommandKind + CommandInvocation）
本质: 描述“用户想干嘛”
特点: 
  - 与输入无关（快捷键 / 菜单 / palette 都能触发）
  - 与 UI 无关
  - 与实现细节无关
  - 可以序列化（宏、日志）

# 编辑器行为（Editor Behavior）
本质：  对“文本世界”的操作
作用对象：
  - buffer
  - selection
  - cursor
  - text structure
  - undo/redo
特点：
  - 完全不关心 UI
  - 不关心 pane / panel / workspace
  - 可单测
  - 可复用

# 工作台行为（Workspace Behavior）
本质：对“IDE 环境”的操作
作用对象：
  - pane
  - tab
  - panel
  - workspace
  - project
  - focus
特点：
  - 不操作文本细节
  - 管理 UI 结构（但不直接画 UI）
  - 决定谁是 active

命令语义是“意图源”，编辑器行为是“对文本的实现”，工作台行为是“对环境的实现”。

# 正确的数据流
Key Event (GPUI)
   ↓
zom-protocol::input
   ↓
CommandInvocation
   ↓
Command Dispatcher
   ↓
├── Editor Handler   (文本操作)
└── Workspace Handler (环境操作)
