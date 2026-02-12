# GPUI-Anim

`gpui-anim` 是一个轻量、流畅的 [GPUI](https://github.com/zed-industries/zed) 框架动画封装库。它旨在以最少的样板代码，为标准 GPUI 元素提供简洁、状态驱动的过渡和动画效果。

> [!WARNING]
>
> 本库目前处于 **早期开发阶段**，API 可能会发生变化。

## ✨ 特性

- **流畅 API**：通过 `.with_transition()` 将任何兼容的 GPUI 元素转换为动画元素
- **零拷贝插值**：高性能的"原地"样式更新，最小化动画帧期间的内存克隆
- **智能过渡**：
  - HSLA 颜色自动最短路径插值（不再有色相跳变！）
  - 支持复杂类型如渐变（Gradient）和尺寸（Size）
- **可组合**：`AnimatedWrapper` 实现了标准 GPUI traits（`Styled`、`ParentElement` 等），可以继续使用熟悉的 GPUI 方法
- **智能回退（持久态恢复）**：
  - **上下文感知恢复**：如果高优先级动画（如 Click）打断了持久状态（如 Hover），系统会"记住"背景状态，并在打断结束后无缝恢复，防止突兀的跳变
- **资源高效（零空转）**：
  - **异步任务休眠**：后台动画 tick 严格事件驱动。当没有活动动画时，任务通过异步通道同步（`recv().await`）进入休眠状态
  - **零 CPU 开销**：线程在空闲时消耗零 CPU 周期，仅在注册新动画时立即唤醒

## 🏗️ 架构设计

`gpui-anim` 采用清晰的分层架构：

```
API 层 (用户接口)
    ↓
核心层 (动画引擎 + 调度器)
    ↓
插值层 (高性能插值计算)
    ↓
过渡曲线层 (缓动函数)
```

详细架构说明请参考 [DESIGN.md](DESIGN.md)。

## 🚀 快速开始

任何实现了 `IntoElement + StatefulInteractiveElement + ParentElement + FluentBuilder + Styled` 的元素都可以被封装。

### 基础用法

```rust
use gpui::*;
use gpui_anim::api::wrapper::TransitionExt;
use gpui_anim::transition::curves::Linear;
use std::time::Duration;

fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div()
        .id("my-animated-box")
        // 使用唯一 ID 初始化动画包装器
        .with_transition("my-animated-box")
        .size_32()
        .bg(rgb(0x2e2e2e))
        // 定义 hover 过渡
        .transition_on_hover(
            Duration::from_millis(300),
            Linear,
            |hovered, state| {
                if *hovered {
                    state.bg(rgb(0xff0000)).size_64()
                } else {
                    state.bg(rgb(0x2e2e2e)).size_32()
                }
            },
        )
}
```

## 🛠 支持的属性

| **类别** | **支持的样式** |
| -------- | -------------- |
| **颜色** | 背景色（纯色、线性渐变）、边框颜色、文字颜色 |
| **布局** | 尺寸（宽度、高度）、最小/最大尺寸、边距、内边距 |
| **视觉** | 透明度、圆角半径、盒阴影 |
| **字体** | 字号、字重 |

## 📖 API 参考

### 初始化

- `.with_transition(id)`：封装元素。需要一个唯一的 `ElementId` 来跨帧追踪动画状态

### 事件驱动动画

这些方法在事件发生时自动触发动画循环：

- `.transition_on_click(duration, transition, modifier)`
- `.transition_on_hover(duration, transition, modifier)`

### 声明式动画

用于响应式状态变化：

- `.transition_when(condition, duration, transition, modifier)`
- `.transition_when_else(condition, duration, transition, then, else_fn)`
- `.transition_when_some(option, ...)` / `.transition_when_none(option, ...)`

### 优先级感知动画

这些变体允许你显式定义过渡的优先级，以解决状态冲突（例如，确保"点击"动画不会被"悬停"状态覆盖）：

- `.transition_on_click_with_priority(duration, transition, priority, modifier)`
- `.transition_on_hover_with_priority(duration, transition, priority, modifier)`
- `.transition_when_with_priority(condition, duration, transition, priority, modifier)`
- `.transition_when_else_with_priority(condition, duration, transition, priority, then, else_fn)`
- `.transition_when_some_with_priority(option, ..., priority, ...)`
- `.transition_when_none_with_priority(option, ..., priority, ...)`

> **优先级级别**：`Lowest`、`Low`、`Medium`、`High`、`Realtime`。高优先级的过渡会覆盖优先级更低或相等的活动动画。

> [!IMPORTANT]
>
> **声明式样式注意事项**：通过 `.transition_when()` 及其变体进行的更改不会自动传播 `App` 上下文。与内部管理上下文的事件监听器不同，你可能需要手动调用刷新（例如 `cx.notify()` 或 `cx.refresh()`）以在外部状态变化时启动过渡。

## 🎨 自定义动画曲线

你不仅限于内置的过渡曲线。你可以通过实现 `Transition` trait 来创建自己的动画曲线（缓动函数）。

### 1. 实现 Trait

只需要实现 `calculate` 方法。它将线性时间进度（$t \in [0, 1]$）映射到你期望的缓动值。

```rust
use gpui_anim::transition::Transition;

pub struct MyCustomBounce;

impl Transition for MyCustomBounce {
    fn calculate(&self, t: f32) -> f32 {
        // 示例：一个简单的平方曲线
        t * t
    }
}
```

### 2. 在 UI 中使用

由于 `Transition` 为 `Arc<T>` 实现了，并且我们提供了 `IntoArcTransition` 辅助工具，你可以直接传递你的结构体：

```rust
div()
    .id("box-1")
    .with_transition("box-1")
    .transition_on_hover(
        Duration::from_millis(500),
        MyCustomBounce, // 你的自定义曲线
        |hovered, state| {
            if *hovered { state.mt_10() } else { state.mt_0() }
        }
    )
    .mt_0()
```

## 🎯 内置过渡曲线

`gpui-anim` 提供了 7 种常用的缓动曲线：

| 曲线名称 | 描述 |
| -------- | ---- |
| `Linear` | 线性过渡（无缓动） |
| `EaseInQuad` | 二次缓进 |
| `EaseOutQuad` | 二次缓出 |
| `EaseInOutQuad` | 二次缓进缓出 |
| `EaseInOutCubic` | 三次缓进缓出 |
| `EaseOutSine` | 正弦缓出 |
| `EaseInExpo` | 指数缓进 |

```rust
use gpui_anim::transition::curves::*;

// 使用不同曲线
.transition_on_hover(duration, EaseInOutCubic, |hovered, state| { ... })
```

## 📚 示例

本仓库包含 6 个完整的示例，展示不同的使用场景：

### 基础示例

```bash
# Hover 动画
cargo run --example hover

# Click 动画
cargo run --example click

# 自定义状态驱动动画
cargo run --example custom_state
```

### 高级示例

```bash
# Hover + Click 组合动画
cargo run --example hover_click_combo

# 多元素不同曲线对比
cargo run --example multi_elements

# 嵌套动画与优先级控制
cargo run --example nested_priority
```

## ⚡ 性能

本库针对高频更新（60/120 FPS）进行了优化：

- **ShadowBackground**：使用 `#[repr(C)]` 内存布局来无开销地插值 GPUI 私有字段
- **FastInterpolatable**：采用原地更新策略，避免每帧完整克隆 `StyleRefinement`
- **初始样式修复**：正确处理 `None → Some` 过渡，避免瞬间跳变
- **智能调度**：120 FPS tick 循环，无动画时自动休眠（零 CPU）
- **rem_size 同步**：与 GPUI runtime 实时同步，确保尺寸插值准确

## 🏛️ 架构亮点

相比原始 `gpui-animation` 项目，`gpui-anim` 提供了：

- ✅ **更清晰的分层**：API / Core / Interpolate / Transition 四层架构
- ✅ **更简洁的命名**：统一 `Anim*` 前缀（`AnimEngine`、`AnimState` 等）
- ✅ **更好的组织**：无冗余占位代码，每个模块职责明确
- ✅ **完善的文档**：架构图、数据流图、职责表一应俱全

详细设计文档请参考 [DESIGN.md](DESIGN.md)。

## 🔧 依赖

```toml
[dependencies]
gpui-anim = { git = "https://github.com/your-repo/gpui-anim" }
```

主要依赖：
- `gpui`: GPUI 框架
- `dashmap`: 并发安全的哈希表
- `parking_lot`: 高性能锁
- `smol`: 异步运行时

## 📝 开发状态

### ✅ 已实现功能

- [x] 流畅的 Fluent API
- [x] 事件驱动动画（hover、click）
- [x] 状态驱动动画（声明式过渡）
- [x] 5 级优先级系统
- [x] 持久态回退机制
- [x] 空闲时零 CPU 消耗
- [x] rem_size 实时同步
- [x] 完整的 GPUI 类型插值支持
- [x] 7 种过渡曲线
- [x] 6 个完整示例

### 🚧 未来计划

- [ ] DirtyFields 缓存（进一步性能优化）
- [ ] 自定义策略插件系统
- [ ] 更多过渡曲线（弹性、回弹等）
- [ ] 序列动画支持
- [ ] 关键帧动画

## 🤝 贡献

欢迎贡献！如果你发现 bug 或有关于新插值支持（如更多布局属性）的建议，请随时开 issue 或提交 pull request。

### 贡献指南

1. Fork 本仓库
2. 创建特性分支（`git checkout -b feature/amazing-feature`）
3. 提交更改（`git commit -m 'Add some amazing feature'`）
4. 推送到分支（`git push origin feature/amazing-feature`）
5. 开启 Pull Request

## 📄 许可证

MIT OR Apache-2.0

## 🙏 致谢

本项目受 [gpui-animation](https://github.com/chi11321/gpui-animation) 启发，在其基础上进行了架构重构和功能增强。

---

**注意**：本项目是学习和实验性质的重构实现。如果你在生产环境中使用，请充分测试。