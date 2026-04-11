# zom-core 是 zom 的“公共语言层”

> 跨 editor、workspace、input、gpui 都成立的稳定抽象。

可以把它理解成：
- zom 的词汇表
- zom 的协议层
- zom 的最小共享模型

## zom-core 应该放些什么？
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
Command、EditorCommand、WorkspaceCommand、可能还有 GlobalCommand。因为这些是全系统共享协议。

3. 输入协议模型
注意，是输入协议，不是输入解析实现。
适合放：
Keystroke
Modifiers
KeyCode
InputContext
FocusTarget
InputResult / InputResolution
因为：
zom-input 要生产/消费它
zom-gpui 要桥接它
zom-workspace 要提供 context facts
zom-editor 可能要读取一部分上下文

4. 标识与元信息类型
例如：
BufferId
PaneId
TabId
WorkspaceId
CommandId
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

## zom-core 的层级划分
zom-core
├── geometry      // 位置、范围、方向
├── selection     // 选区模型
├── command       // 命令语义
├── input         // 输入协议
├── ids           // 强类型 ID
└── state         // 少量通用状态类型

# 命令语义（Command）
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

命令语义是“意图”，编辑器行为是“对文本的实现”，工作台行为是“对环境的实现”。

# 正确的数据流
Key Event (GPUI)
   ↓
zom-input
   ↓
Command
   ↓
Command Dispatcher
   ↓
├── Editor Handler   (文本操作)
└── Workspace Handler (环境操作)
