# Crate Boundary Checklist

版本：`v0.6`  
适用范围：`/Users/liuyuan/project/zom` workspace 内部 crate 依赖关系（只讨论 workspace crate 之间的直接依赖）

## 1. 目标

这份清单是“crate 边界约束”的执行版，专门回答两个问题：

1. 每一层 **允许** 依赖哪些 crate？
2. 每一层 **禁止** 依赖哪些 crate？

评审时若与其他文档冲突，以本清单和边界校验脚本为准。

## 2. 当前单向依赖拓扑

```text
=======================================================================
                      ZOM 模块依赖与架构分层图
=======================================================================

【 L1: 应用入口层 】
                       apps/zom-desktop
                              │
                              ▼
【 L2: 界面与框架层 】
                       crates/zom-gpui
                              │
                              ▼
【 L3: 运行时引擎层 】
                      crates/zom-runtime
                              │
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
【 L4: 核心业务层 】          │                   │
  zom-workspace           zom-input           zom-editor
                                                  │
                                                  ▼ (依赖)
【 L4.5: 领域层 】                            zom-text


┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈
【 L5: 基建与协议层 】 (全局共享底座)

      [ crates/zom-protocol ]       [ crates/zom-text-tokens ]

  * 附注（为了图表清晰，省略了连接到此层的连线）：
    - L2 (gpui) 依赖 L5 两个库
    - L3 (runtime) 依赖 L5 两个库
    - L4 所有的模块均依赖 protocol，editor/text 还额外依赖 tokens。
=======================================================================
```

补充说明：

1. 上图表达的是 workspace 内部 crate 的直接依赖关系（`A -> B` 表示 `A` 直接依赖 `B`）。
2. `zom-workspace` 当前是可选承载层，但一旦引用，仍必须遵守单向依赖。
3. `zom-runtime` 允许依赖 `zom-workspace`，但不是强制。
4. 外部第三方依赖不在本表内；本表只约束 workspace 内部 crate 间关系。

## 3. Allowed / Forbidden 依赖总表

| Crate | Allowed（仅 workspace 内） | Forbidden（仅 workspace 内） |
|---|---|---|
| `zom-text-tokens` | 无 | `zom-protocol`、`zom-input`、`zom-text`、`zom-editor`、`zom-workspace`、`zom-runtime`、`zom-gpui`、`zom-desktop` |
| `zom-protocol` | 无 | `zom-text-tokens`、`zom-input`、`zom-text`、`zom-editor`、`zom-workspace`、`zom-runtime`、`zom-gpui`、`zom-desktop` |
| `zom-input` | `zom-protocol` | `zom-text-tokens`、`zom-text`、`zom-editor`、`zom-workspace`、`zom-runtime`、`zom-gpui`、`zom-desktop` |
| `zom-text` | `zom-protocol`、`zom-text-tokens` | `zom-input`、`zom-editor`、`zom-workspace`、`zom-runtime`、`zom-gpui`、`zom-desktop` |
| `zom-editor` | `zom-protocol`、`zom-text`、`zom-text-tokens` | `zom-input`、`zom-workspace`、`zom-runtime`、`zom-gpui`、`zom-desktop` |
| `zom-workspace` | `zom-protocol` | `zom-text-tokens`、`zom-input`、`zom-text`、`zom-editor`、`zom-runtime`、`zom-gpui`、`zom-desktop` |
| `zom-runtime` | `zom-protocol`、`zom-input`、`zom-text`、`zom-editor`、`zom-workspace`、`zom-text-tokens` | `zom-gpui`、`zom-desktop` |
| `zom-gpui` | `zom-protocol`、`zom-runtime`、`zom-text-tokens` | `zom-input`、`zom-text`、`zom-editor`、`zom-workspace`、`zom-desktop` |
| `zom-desktop` | `zom-gpui` | `zom-text-tokens`、`zom-protocol`、`zom-input`、`zom-text`、`zom-editor`、`zom-workspace`、`zom-runtime` |

## 4. 层内附加约束

1. 禁止循环依赖（direct 或 transitive）。
2. `zom-protocol` 保持协议层定位，禁止引入 UI/平台原始事件/业务状态机。
3. `zom-gpui` 只做事件适配与渲染，禁止承载文本内核算法。
4. `zom-runtime` 负责编排，不反向要求 `zom-gpui` 暴露 UI 实现细节。
5. 跨 crate 共享语义类型优先沉淀到 `zom-protocol`，不要在上层重复定义 1:1 镜像类型。

### 4.1 边界执行纪律（强制）

1. 开发前必须先判定主 crate 与职责归属，禁止先实现后迁移。
2. 禁止以“先跑通”为理由在错误层放实现；职责不明确时先补 SoT 文档并在 PR 中说明。
3. 禁止跨层重复定义已存在的稳定类型或能力（含 1:1 镜像结构与重复实现）。
4. 一旦发现职责漂移，必须先回收边界，再继续叠加功能。
5. 评审发现边界问题可直接拒绝合入，不以“功能可用”作为放行理由。

## 5. 变更与验收流程

1. 若新增/调整依赖方向，必须同步更新：
   - 本文档；
   - `docs/architecture/架构哲学与模块宪章.md`（若涉及治理/职责变化）；
   - `scripts/check-boundaries.sh`；
   - PR 中的“架构变化”字段（见 `.github/pull_request_template.md`）。
2. PR 合入前必须执行：

```bash
./scripts/check-boundaries.sh
./scripts/check-control-char-literals.sh
./scripts/check-rustdoc-coverage.sh
cargo clippy --workspace --all-targets -- -D warnings
cargo check
```

3. PR 描述必须包含“主 crate（唯一）+ 次要 crate + 是否触及 `zom-protocol` + 是否存在职责漂移”。

## 6. 关联文档

1. [docs 导航与分工](../README.md)
2. [架构哲学与模块宪章](./架构哲学与模块宪章.md)
3. [开发规范手册](../standards/开发规范手册.md)
4. [面板开发契约](../playbooks/面板开发契约.md)
5. [PR 模板](../../.github/pull_request_template.md)
