# DESIGN

本文档为 `gpui-anim` 的详细设计说明。该库以 `gpui-animation` 为参考，实现相同功能，但具备更清晰的分层、更强的可扩展性以及更稳健的性能策略。本文还定义了命名规范，约定将冗长的 `Animation*` 命名简化为 `Anim*`。

---

## 1. 目标与非目标

### 1.1 目标
- **结构清晰**：API 层与核心引擎、插值模块解耦。
- **性能稳定**：支持高频动画（60/120 FPS）且低开销。
- **可扩展性**：易于新增过渡曲线、插值类型与调度策略。
- **易测试**：核心动画逻辑可在无 UI 环境下单测。

### 1.2 非目标
- 不尝试替代 GPUI 的渲染与事件系统。
- 不提供跨框架（非 GPUI）支持。
- 不追求零 unsafe，unsafe 必须集中、可审计。

---

## 2. 项目架构设计

### 2.1 架构分层

`gpui-anim` 采用清晰的三层架构：

```
┌─────────────────────────────────────────────────────────────┐
│                       API 层 (api/)                         │
│  • AnimatedWrapper: Fluent API 包装器                       │
│  • TransitionExt: trait 扩展，提供 with_transition()        │
│  • AnimEvent/AnimPriority: 公共类型定义                     │
│  职责：用户接口、事件绑定、声明式动画 API                   │
└─────────────────────────────────────────────────────────────┘
                           ↓ AnimRequest
┌─────────────────────────────────────────────────────────────┐
│                      核心层 (core/)                         │
│  • AnimEngine: 动画引擎（状态管理 + 全局单例）              │
│  • AnimScheduler: 帧调度器（120fps tick + 空闲休眠）        │
│  • AnimState: 动画状态机（from/to/cur + 版本控制）          │
│  • Policies: 优先级与中断策略                               │
│  • Metrics: rem_size 全局同步                               │
│  职责：状态推进、优先级判断、持久态回退、tick 驱动          │
└─────────────────────────────────────────────────────────────┘
                           ↓ 插值请求
┌─────────────────────────────────────────────────────────────┐
│                    插值层 (interpolate/)                    │
│  • Interpolatable/FastInterpolatable: 插值 trait            │
│  • gpui_adapters: GPUI 类型插值（StyleRefinement 等）       │
│  • generic: 通用类型插值（f32、Pixels 等）                  │
│  职责：高性能插值计算、GPUI 类型适配、unsafe 隔离           │
└─────────────────────────────────────────────────────────────┘
                           ↑ Transition
┌─────────────────────────────────────────────────────────────┐
│                   过渡曲线 (transition/)                    │
│  • Transition trait: 时间映射接口                           │
│  • curves: Linear + 6 种 easing 函数                        │
│  职责：时间曲线计算、缓动函数                               │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 数据流动

```
用户代码
   ↓ .with_transition("id").transition_on_hover(...)
API 层（AnimatedWrapper）
   ↓ 生成 AnimRequest
核心层（AnimEngine）
   ↓ 创建/更新 AnimState
   ↓ 注册到 ActiveAnim
调度器（AnimScheduler）
   ↓ 120fps tick 循环
   ↓ 调用 engine.tick()
AnimState.animated()
   ↓ 计算 progress = transition.run(...)
   ↓ 调用 fast_interpolate(from, to, progress, &mut cur)
插值层
   ↓ 更新 StyleRefinement.cur
渲染
   ↓ root.style().refine(&state.cur)
GPUI 渲染系统
```

### 2.3 核心模块职责

| 模块 | 文件 | 核心类型 | 职责 |
|------|------|----------|------|
| **API** | `api/wrapper.rs` | `AnimatedWrapper` | Fluent API、事件绑定 |
| | `api/types.rs` | `AnimEvent`, `AnimPriority` | 公共类型定义 |
| **核心** | `core/engine.rs` | `AnimEngine`, `AnimRequest` | 状态存储、动画提交 |
| | `core/scheduler.rs` | `AnimScheduler` | 帧调度、空闲等待 |
| | `core/state.rs` | `AnimState<T>` | 动画状态机 |
| | `core/policies.rs` | `PriorityPolicy`, `InterruptionPolicy` | 策略接口 |
| | `core/metrics.rs` | `rem_size()` | 全局度量 |
| **插值** | `interpolate/traits.rs` | `Interpolatable`, `FastInterpolatable` | 插值 trait |
| | `interpolate/gpui_adapters.rs` | GPUI 类型实现 | GPUI 适配层 |
| | `interpolate/generic.rs` | `f32` 等实现 | 通用插值 |
| **过渡** | `transition.rs` | `Transition` trait | 曲线接口 |
| | `transition/curves.rs` | `Linear`, `EaseInQuad` 等 | 7 种曲线 |

### 2.4 命名规范

- **前缀统一**：动画相关类型使用 `Anim*` 前缀（`AnimState`, `AnimEngine` 等）
- **模块名简洁**：使用单文件入口（`core.rs` 而非 `core/mod.rs`）
- **类型清晰**：避免冗长的 `Animation*` 全名

---

## 3. 实际项目结构

采用 Rust 当前最佳实践：**使用与模块同名的入口文件**（如 `api.rs`、`core.rs`），避免 `mod.rs`。

```
gpui-anim/
  src/
    lib.rs
    api.rs
    api/
      wrapper.rs          // AnimatedWrapper + TransitionExt
      types.rs            // AnimPriority、AnimEvent
    core.rs
    core/
      engine.rs           // AnimEngine + 全局单例
      scheduler.rs        // AnimScheduler：帧调度 + 空闲等待
      state.rs            // AnimState：动画状态
      policies.rs         // 优先级与中断策略
      metrics.rs          // rem_size 全局指标
    interpolate.rs
    interpolate/
      traits.rs           // Interpolatable / FastInterpolatable
      gpui_adapters.rs    // GPUI 类型插值适配（含 unsafe）
      generic.rs          // f32 等通用类型插值
    transition.rs
    transition/
      curves.rs           // Linear + 6 种 easing 曲线
  examples/
    hover.rs              // hover 动画示例
    click.rs              // click 动画示例
    custom_state.rs       // 状态驱动动画示例
```

### 与设计初稿的差异
- **合并优化**：`AnimRegistry` 并入 `AnimEngine`，全局单例直接在 `engine.rs`
- **简化模块**：移除未使用的 `events.rs` 和 `cache.rs` 占位模块
- **曲线集中**：所有过渡曲线在单个 `curves.rs` 文件，不再拆分子文件夹
- **新增 metrics**：独立的 `metrics.rs` 管理 rem_size 同步

---

## 4. API 设计

### 4.1 API 层职责
- 提供 Fluent 链式接口。
- 负责事件绑定与状态驱动入口。
- 将用户输入转换为 `AnimRequest`。

### 4.2 API 约束
- `with_transition(id)` 仅初始化包装器，不直接修改状态。
- `transition_on_*` 与 `transition_when_*` 只负责“声明意图”，实际执行交给 `AnimEngine`。
- declarative API 需要显式触发刷新（由用户或框架回调）。

### 4.3 公开类型示例
- `AnimPriority`
- `AnimEvent`
- `Transition`
- `TransitionExt`

---

## 5. 核心状态模型

### 5.1 AnimState
```
AnimState<T> {
  origin: T,
  from: T,
  to: T,
  cur: T,
  progress: f32,
  start_at: Instant,
  version: usize,
  priority: AnimPriority,
}
```

### 5.2 AnimRequest
```
AnimRequest {
  id: ElementId,
  event: AnimEvent,
  duration: Duration,
  transition: Arc<dyn Transition>,
  priority: AnimPriority,
  modifier: FnOnce(AnimState<StyleRefinement>) -> AnimState<StyleRefinement>,
  persistent: bool,
}
```

---

## 6. 核心模块设计

### 6.1 AnimEngine（`core/engine.rs`）
职责：
- 接收 `AnimRequest`。
- 生成或更新 `AnimState`。
- 决定是否启动调度 tick。

核心接口：
- `AnimEngine::submit(request)`
- `AnimEngine::tick(delta_time)`

### 6.2 AnimScheduler（`core/scheduler.rs`）
职责：
- 驱动 `tick`。
- 在无活动动画时休眠。
- 支持可配置 FPS（60/120）。

### 6.3 全局引擎（`core/engine.rs`）
职责：
- `AnimEngine` 内部使用 `DashMap` 存储状态和活动动画。
- 通过 `LazyLock` 提供全局单例访问。
- 支持并发安全的 `submit()` 和 `tick()`。

### 6.4 Policies（`core/policies.rs`）
职责：
- 定义优先级与中断策略。
- 支持用户自定义策略实现。

示例：
- `PriorityPolicy`
- `InterruptionPolicy`

---

## 7. 插值模块设计

### 7.1 插值 trait
- `Interpolatable`: 返回新值。
- `FastInterpolatable`: in-place 更新。

### 7.2 GPUI 类型适配
- 将 GPUI 私有布局转换放到 `gpui_adapters.rs`。
- unsafe 仅集中在此文件，便于审计。

### 7.3 DirtyFields 缓存
- 在动画启动时预计算变动字段集合。
- 每帧只对变动字段进行插值。
- 可用 bitset 或 enum 集合作为实现。

---

## 8. 性能策略

### 8.1 状态存储策略
- **已采用**：`DashMap`（并发安全，适合 GPUI 多窗口场景）
- 内部无锁迭代优化：先收集快照再推进状态

### 8.2 插值优化
- 使用 `FastInterpolatable` 直接更新 `cur`。
- 避免在每帧 clone `StyleRefinement`。

### 8.3 过渡曲线缓存
- 可对频繁使用的曲线使用 `Arc` 共享。
- 对固定曲线（Linear 等）可提供静态实例。

### 8.4 帧率自适应
- 若可获取刷新率则同步。
- 否则使用配置值（默认 120）。

---

## 9. 事件与优先级策略

### 9.1 AnimEvent
- `None`
- `Hover`
- `Click`
- `Custom(...)`

### 9.2 AnimPriority
- `Lowest`
- `Low`
- `Medium`
- `High`
- `Realtime`

### 9.3 中断原则
- 高优先级可打断低优先级。
- 支持持久态恢复（hover → click → hover 恢复）。

---

## 10. 文档与测试

### 10.1 文档结构建议
- 设计概览
- 使用指南
- API 分类（事件驱动 / 状态驱动）
- 性能原理
- 过渡曲线扩展
- 常见问题

### 10.2 测试策略
- `core` 逻辑单测：提交 `AnimRequest` 断言状态。
- 插值测试：
  - 色相最短路径
  - Rems/Pixels 互转
  - DirtyFields 命中率

---

## 11. 实现里程碑

✅ **已完成（当前版本）**：
1. **核心功能**：Fluent API + Hover/Click + 状态驱动动画
2. **插值系统**：FastInterpolatable + 完整 GPUI 类型支持
3. **性能优化**：空闲等待 + rem_size 同步 + 初始样式修复
4. **策略系统**：优先级策略 + 持久态回退
5. **过渡曲线**：7 种常用 easing 函数
6. **示例验证**：3 个完整示例覆盖主要用例

🔮 **未来扩展**：
- DirtyFields 缓存（进一步性能优化）
- 自定义策略插件系统
- 更多过渡曲线（弹性、回弹等）

---

## 12. 总结

`gpui-anim` 将以更清晰的分层、更简洁的命名、更可控的性能策略，达到与 `gpui-animation` 同等功能但更高可维护性的目标。通过 `Anim*` 命名统一和模块化设计，可让未来的扩展与维护成本显著降低。
