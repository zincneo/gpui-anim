# gpui-anim 重构需求文档

## 一、项目概述

`gpui-anim` 是一个为 [GPUI](https://github.com/zed-industries/zed) 框架提供声明式动画能力的扩展库。目标是让 GPUI 元素能够以简洁的链式调用方式获得平滑的样式过渡动画。

---

## 二、现存问题分析

### 2.1 核心架构问题

#### P0 — `on_hover` / `on_click` 单次调用冲突

**现状**：`AnimatedWrapper` 在 `RenderOnce::render()` 内部调用了内部元素的 `.on_hover()` 和 `.on_click()`。由于 GPUI 的 `StatefulInteractiveElement` 限制这两个方法只能各调用一次，导致：

- 用户无法在 `.with_transition()` 之后再对元素调用 `.on_hover()` / `.on_click()`
- `AnimatedWrapper` 自身提供的 `.on_hover()` 只是存储回调在内部转发，但该 API 命名与 GPUI 原生方法完全相同，语义混淆
- 如果用户在 `.with_transition()` 之前已经注册了 hover/click 处理器，会被覆盖

**根本原因**：AnimatedWrapper 将"事件监听"和"动画驱动"耦合在一起，侵占了元素仅有的一次事件注册机会。

#### P1 — 全局单例引擎 (Global Singleton)

- `AnimEngine` 通过 `LazyLock<AnimEngine>` 作为全局静态变量存在
- `AnimScheduler` 同样是全局 `LazyLock`
- 全局状态导致：无法编写单元测试、无法在多窗口场景下隔离状态、无法重置引擎

#### P2 — 模块层级过深

当前目录结构对于一个约 1200 行代码的库来说嵌套过深：

```
src/
  api/types.rs          # 仅定义 AnimEvent + AnimPriority (~30 行)
  api/wrapper.rs        # 核心 wrapper 代码
  core/engine.rs        # 动画引擎
  core/metrics.rs       # 仅维护一个 rem_size 全局变量
  core/policies.rs      # 2 个 trait + 2 个默认实现 (~40 行)
  core/scheduler.rs     # 调度循环
  core/state.rs         # AnimState
  interpolate/generic.rs       # f32 插值 (~15 行)
  interpolate/gpui_adapters.rs # GPUI 类型插值
  interpolate/traits.rs        # 2 个 trait (~15 行)
  transition/curves.rs  # 缓动曲线
```

许多文件内容极少（< 40 行），拆成子模块反而增加了认知成本和 import 路径长度。

#### P3 — API 组合爆炸

几乎每个 `transition_*` 方法都有一个 `_with_priority` 变体，导致公开 API 数量翻倍：

- `transition_on_hover` / `transition_on_hover_with_priority`
- `transition_on_click` / `transition_on_click_with_priority`
- `transition_when` / `transition_when_with_priority`
- `transition_when_else` / `transition_when_else_with_priority`
- `transition_when_some` / `transition_when_some_with_priority`
- `transition_when_none` / `transition_when_none_with_priority`

这些方法签名几乎完全一致，仅多一个 `priority` 参数。

#### P4 — `unsafe` 使用风险

1. **`metrics.rs`**：`Pixels` ↔ `f32` 使用 `std::mem::transmute`，如果 `Pixels` 的内存布局变更将导致未定义行为
2. **`gpui_adapters.rs`**：`ShadowBackground` 是对 `Background` 的 `#[repr(C)]` 影子结构体，通过裸指针强转实现互操作。一旦 GPUI 上游修改 `Background` 布局，将静默产生内存安全问题
3. **`gpui_adapters.rs`**：`Pixels` 的 `interpolate` 也使用 `transmute`

#### P5 — 动画状态永不清理

`AnimEngine.states` 是一个 `DashMap<ElementId, AnimState>`，元素被销毁后其对应的状态条目不会被移除，在长时间运行的应用中会造成内存泄漏。

---

### 2.2 设计缺陷

#### D1 — AnimState 与 StyleRefinement 强耦合

`AnimState<T>` 虽然使用了泛型，但 engine、wrapper 层全部硬编码为 `AnimState<StyleRefinement>`，泛型形同虚设。如果未来需要动画化非样式属性（如 transform、自定义数值），当前架构不支持。

#### D2 — Transition trait 职责不清

`Transition::run()` 方法同时负责"计算经过时间比例"和"应用缓动函数"两个职责。时间计算逻辑应该由引擎统一管理，Transition 只需关心 `calculate(t) -> f32`。

#### D3 — IntoArcTransition 无必要

`IntoArcTransition<T>` trait 的唯一作用是允许用户传递 `T` 或 `Arc<T>`。然而所有缓动曲线都是 ZST (零大小类型，如 `Linear`、`EaseInQuad`)，`Arc` 包装完全多余。可以直接接受 `impl Transition` 并在内部 Arc 化。

#### D4 — 缺少生命周期管理

没有机制在元素不再渲染时自动回收动画状态。持久动画（`persistent: true`）的生命周期管理完全依赖手动标记。

#### D5 — 调度器 FPS 硬编码

`DEFAULT_FPS: f32 = 120.0` 硬编码在 scheduler 中，不可配置，也无法适配不同刷新率的显示器。

---

### 2.3 用户体验问题

#### U1 — import 路径过长

用户需要写：

```rust
use gpui_anim::api::wrapper::TransitionExt;
use gpui_anim::transition::curves::Linear;
```

理想情况应该是：

```rust
use gpui_anim::prelude::*;
```

#### U2 — hover 动画需要手动处理两个方向

当前 `transition_on_hover` 的回调签名是 `Fn(&bool, StyleState) -> StyleState`，用户必须自己检查 `*hovered` 并分别处理进入和离开两个方向。对于最常见的场景（hover 时应用某些样式，离开时恢复），这是不必要的模板代码。

#### U3 — `with_transition` 需要额外传 id

```rust
div().id("box").with_transition("box")  // id 重复传递
```

元素已经有了 `id`，但 `with_transition` 要求再传一次，容易出错且冗余。

---

## 三、重构方案

### 3.1 解决 hover/click 冲突（P0）

**方案：将动画逻辑从事件监听中剥离**

不再在内部抢占 `on_hover()` / `on_click()`，改为：

1. **方案 A — Element decorator 模式**：在 `RenderOnce` 中不调用内部元素的 `on_hover`/`on_click`，而是将 AnimatedWrapper 自身作为外层容器，在外层容器上监听交互事件。内部元素保留完整的交互能力。

```rust
// 实现草案
impl RenderOnce for AnimatedWrapper<E> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // 外层 div 负责动画的 hover/click 检测
        // 内层元素保持原样，用户可以自由注册 on_hover/on_click
        div()
            .id(self.anim_id.clone())
            .on_hover(move |hovered, window, cx| {
                // 提交动画请求
            })
            .on_click(move |event, window, cx| {
                // 提交动画请求
            })
            .child(self.child)  // 内部元素不受影响
    }
}
```

2. **方案 B — 基于 GPUI observe/subscribe 机制**：利用 GPUI 的 Model 来存储 hover/click 状态，通过 `cx.observe()` 监听状态变化来触发动画，与元素的原生交互能力完全解耦。

**推荐方案 A**，因为实现更简单，且对用户 API 影响最小。

### 3.2 消除全局单例（P1）

将引擎实例存入 GPUI 的 `App` 全局状态中：

```rust
// 使用 GPUI 的 Global trait
impl Global for AnimEngine {}

// 初始化
cx.set_global(AnimEngine::new());

// 访问
cx.global::<AnimEngine>()
```

这样：

- 每个 App 实例拥有独立的引擎
- 测试时可以创建独立的上下文
- 无需 `LazyLock` + `DashMap`，因为 GPUI 的 `App` 已经提供了线程安全保证

### 3.3 扁平化模块结构（P2）

```
src/
  lib.rs          # pub mod + prelude
  prelude.rs      # 统一的公开导出
  anim.rs         # AnimatedWrapper + TransitionExt (原 api/wrapper.rs)
  engine.rs       # AnimEngine + AnimRequest (原 core/engine.rs)
  state.rs        # AnimState (原 core/state.rs)
  scheduler.rs    # AnimScheduler (原 core/scheduler.rs)
  interpolate.rs  # 所有插值 trait + 实现 (合并 interpolate/*)
  transition.rs   # Transition trait + 所有缓动曲线 (合并 transition/*)
  types.rs        # AnimEvent, AnimPriority 等公共类型
```

合并原则：

- 将 `interpolate/traits.rs`、`interpolate/generic.rs`、`interpolate/gpui_adapters.rs` 合并为单个 `interpolate.rs`
- 将 `transition/curves.rs` 和 `transition.rs` 合并
- 将 `core/metrics.rs` 内联到需要的地方（或移入 `engine.rs`）
- 将 `core/policies.rs` 内联到 `engine.rs`（代码量极少）
- 移除 `api/` 目录层级

### 3.4 精简 API — Builder 模式（P3）

引入 `TransitionBuilder` 将可选参数链式设置，消除 `_with_priority` 变体：

```rust
// Before (当前)
.transition_on_hover_with_priority(
    Duration::from_millis(200),
    EaseOutQuad,
    AnimPriority::High,
    |hovered, state| { ... }
)

// After (重构后)
.anim_on_hover(Duration::from_millis(200), EaseOutQuad, |hovered, state| { ... })
    .priority(AnimPriority::High)
```

或者更简洁，让 priority 成为可选参数使用默认值，只在需要时覆盖：

```rust
.transition_on_hover(
    Duration::from_millis(200),
    EaseOutQuad,
    |hovered, state| { ... }
)
.with_priority(AnimPriority::High)  // 可选
```

### 3.5 消除 unsafe（P4）

1. **Pixels ↔ f32**：使用 `Pixels::0` 字段或 `Into<f32>` / `From<f32>` 转换，不使用 `transmute`
2. **ShadowBackground**：移除影子结构体，改为为 `Background` 的每个变体分别实现插值逻辑，或者使用 GPUI 提供的公开 API 访问字段
3. **rem_size 全局变量**：如果引擎移入 GPUI Global，rem_size 可以直接从 Window 获取，不再需要 AtomicU32 hack

### 3.6 添加 prelude（U1）

```rust
// src/prelude.rs
pub use crate::anim::{AnimatedWrapper, TransitionExt};
pub use crate::transition::*;
pub use crate::types::{AnimEvent, AnimPriority};
```

用户只需：

```rust
use gpui_anim::prelude::*;
```

### 3.7 简化 hover API（U2）

为最常见的 hover 场景提供便捷方法：

```rust
// 方式1：只需指定 hover 时的样式，离开自动恢复
.hover_style(Duration::from_millis(200), EaseOutQuad, |state| {
    state.bg(rgb(0xff0000)).size_64()
})

// 方式2：需要区分进入/离开时的完整控制
.transition_on_hover(Duration::from_millis(200), EaseOutQuad, |hovered, state| {
    if *hovered { state.bg(rgb(0xff0000)) } else { state.origin() }
})
```

### 3.8 自动推断动画 ID（U3）

从内部元素提取已有的 `ElementId`，避免重复传递：

```rust
// Before
div().id("box").with_transition("box")

// After - 方案1: 自动从元素获取 id
div().id("box").animated()

// After - 方案2: 只在需要不同 id 时传递
div().id("box").animated_as("custom-anim-id")
```

### 3.9 动画状态清理（P5）

引入引用计数或代际(generation)机制：

- 每次 `render()` 时标记该 ElementId 为"活跃"
- 调度器在 tick 中检查未活跃超过 N 帧的条目并清理
- 或者利用 GPUI 的 `Drop` 语义，在 wrapper 销毁时通知引擎回收

### 3.10 调度器改进（D5）

- 允许配置目标 FPS：`AnimEngine::new().with_fps(60)`
- 考虑使用 GPUI 自身的帧回调（如 `window.on_next_frame`）替代独立的 `smol::Timer` 循环，以与渲染管线同步

---

## 四、重构后的用户 API 愿景

```rust
use gpui_anim::prelude::*;

impl Render for MyComponent {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("my-box")
            .size_32()
            .bg(rgb(0x2e2e2e))
            // 转换为动画元素 — 自动使用元素的 id
            .animated()
            // hover 动画 — 不抢占 on_hover，用户仍可在外层注册 on_hover
            .hover_style(Duration::from_millis(200), EaseOutQuad, |state| {
                state.bg(rgb(0xff0000)).size_64()
            })
            // click 动画
            .transition_on_click(Duration::from_millis(300), EaseInOutCubic, |_event, state| {
                state.bg(rgb(0x00ff00))
            })
            // 条件动画
            .transition_when(self.is_active, Duration::from_millis(500), Linear, |state| {
                state.opacity(1.0)
            })
            // 用户仍然可以使用原生 on_hover
            .on_hover(|hovered, window, cx| {
                println!("hovered: {}", hovered);
            })
            .child("Hello")
    }
}
```

---

## 五、实施优先级

| 阶段        | 任务                                    | 优先级      | 影响                  |
| ----------- | --------------------------------------- | ----------- | --------------------- |
| **Phase 1** | 修复 on_hover/on_click 冲突 (P0)        | 🔴 Critical | 解除用户核心限制      |
| **Phase 1** | 添加 prelude 统一导出 (U1)              | 🟡 Medium   | 改善用户体验          |
| **Phase 2** | 扁平化模块结构 (P2)                     | 🟡 Medium   | 降低代码认知成本      |
| **Phase 2** | 精简 API 消除 \_with_priority 变体 (P3) | 🟡 Medium   | 减少 API 表面积       |
| **Phase 2** | 简化 hover API (U2) + 自动推断 id (U3)  | 🟡 Medium   | 改善用户体验          |
| **Phase 3** | 消除全局单例 (P1)                       | 🟠 High     | 可测试性 + 多窗口支持 |
| **Phase 3** | 消除 unsafe (P4)                        | 🟠 High     | 内存安全              |
| **Phase 3** | 动画状态清理 (P5)                       | 🟠 High     | 防止内存泄漏          |
| **Phase 4** | 调度器改进 (D5)                         | 🟢 Low      | 性能优化              |
| **Phase 4** | AnimState 泛型化 (D1)                   | 🟢 Low      | 扩展能力              |
| **Phase 4** | 清理 Transition trait (D2, D3)          | 🟢 Low      | 代码整洁              |

---

## 六、关于 hover 问题的详细分析

### 问题复现

```rust
div()
    .id("box")
    .with_transition("box")
    .transition_on_hover(...)
    .on_hover(|_, _, _| { /* 用户自己的 hover 逻辑 */ })
    //       ^^^^^^^^ 编译错误：on_hover 只能调用一次
```

### 根因链路

1. `AnimatedWrapper` 的 `RenderOnce::render()` 实现中（[src/api/wrapper.rs](src/api/wrapper.rs) 第 165-195 行）：
   ```rust
   root.on_hover(move |hovered, window, app| {
       // 转发用户回调 + 提交动画请求
   })
   .on_click(move |event, window, app| {
       // 转发用户回调 + 提交动画请求
   })
   ```
2. GPUI 的 `on_hover` 在 trait `InteractiveElement` 中被实现为只能调用一次
3. `AnimatedWrapper` 虽然提供了自己的 `on_hover` 方法来"转发"用户回调，但这个 API 只是将回调存储在字段中，再在内部的 GPUI `on_hover` 里调用。用户仍然**无法在 AnimatedWrapper 之外**对同一个元素注册 `on_hover`

### 解决方案详述

采用 **外层容器 (decorator) 模式**：

```rust
impl<E> RenderOnce for AnimatedWrapper<E> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut inner = self.child;        // 用户的元素，保持原样

        // 计算当前动画状态并应用样式
        if let Some(state) = engine.state(&self.id) {
            inner.style().refine(&state.cur);
        }

        // 外层容器仅负责检测 hover/click 并驱动动画
        // 内部元素的 on_hover/on_click 完全不被触碰
        div()
            .id(self.anim_id)
            .on_hover(move |hovered, window, cx| {
                // 仅提交动画请求，不干涉内部元素
                submit_hover_transition(...)
            })
            .on_click(move |event, window, cx| {
                submit_click_transition(...)
            })
            .child(inner.children(self.children))
    }
}
```

**优势**：

- 内部元素的 `on_hover` / `on_click` 完全不受影响
- 用户可以在 `.animated()` 之前或之后自由注册交互回调
- 动画的事件检测与用户的事件处理完全分离

**注意事项**：

- 外层容器需要设置合适的布局属性，避免影响内部元素的尺寸/定位
- 需要处理事件冒泡，确保外层捕获的 hover 状态与内部元素一致
