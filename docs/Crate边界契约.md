# Crate Boundary Checklist

版本：`v0.1`  
适用范围：`/Users/liuyuan/project/zom` workspace 内部 crate 依赖关系（只讨论 workspace crate 之间的直接依赖）

## 1. 目标

这份清单是“crate 边界约束”的执行版，专门回答两个问题：

1. 每一层 **允许** 依赖哪些 crate？
2. 每一层 **禁止** 依赖哪些 crate？

评审时若与其他文档冲突，以本清单和边界校验脚本为准。

## 2. 当前单向依赖拓扑

```text
zom-protocol
  ↑
zom-text      zom-editor
   ↑            ↑
   └─────── zom-workspace
             ↑
          zom-runtime
             ↑
          zom-gpui
             ↑
        apps/zom-desktop
```

补充说明：

1. `zom-workspace` 当前是可选承载层，但一旦引用，仍必须遵守单向依赖。
2. `zom-runtime` 允许依赖 `zom-workspace`，但不是强制。
3. 外部第三方依赖不在本表内；本表只约束 workspace 内部 crate 间关系。

## 3. Allowed / Forbidden 依赖总表

| Crate | Allowed（仅 workspace 内） | Forbidden（仅 workspace 内） |
|---|---|---|
| `zom-protocol` | 无 | `zom-text`、`zom-editor`、`zom-workspace`、`zom-runtime`、`zom-gpui`、`zom-desktop` |
| `zom-text` | `zom-protocol` | `zom-editor`、`zom-workspace`、`zom-runtime`、`zom-gpui`、`zom-desktop` |
| `zom-editor` | `zom-protocol`、`zom-text` | `zom-workspace`、`zom-runtime`、`zom-gpui`、`zom-desktop` |
| `zom-workspace` | `zom-protocol`、`zom-text`、`zom-editor` | `zom-runtime`、`zom-gpui`、`zom-desktop` |
| `zom-runtime` | `zom-protocol`、`zom-text`、`zom-editor`、`zom-workspace` | `zom-gpui`、`zom-desktop` |
| `zom-gpui` | `zom-protocol`、`zom-runtime` | `zom-text`、`zom-editor`、`zom-workspace`、`zom-desktop` |
| `zom-desktop` | `zom-runtime`、`zom-gpui` | `zom-protocol`、`zom-text`、`zom-editor`、`zom-workspace` |

## 4. 层内附加约束

1. 禁止循环依赖（direct 或 transitive）。
2. `zom-protocol` 保持协议层定位，禁止引入 UI/平台原始事件/业务状态机。
3. `zom-gpui` 只做事件适配与渲染，禁止承载文本内核算法。
4. `zom-runtime` 负责编排，不反向要求 `zom-gpui` 暴露 UI 实现细节。
5. 跨 crate 共享语义类型优先沉淀到 `zom-protocol`，不要在上层重复定义 1:1 镜像类型。

## 5. 变更与验收流程

1. 若新增/调整依赖方向，必须同步更新：
   - 本文档；
   - `scripts/check-boundaries.sh`；
   - 必要时新增 ADR 记录（`docs/adr/`）。
2. PR 合入前必须执行：

```bash
./scripts/check-boundaries.sh
cargo check
```

## 6. 关联文档

1. [开发规范手册](./开发规范手册.md)
2. [面板开发契约](./面板开发契约.md)
3. [ADR 0001：工作台命令与输入路由收敛](./adr/0001-workspace-command-and-input-routing.md)
