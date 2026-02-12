//! Animated wrapper API for gpui-anim.

use std::sync::Arc;
use std::time::Duration;

use gpui::{prelude::FluentBuilder, *};

use crate::api::types::{AnimEvent, AnimPriority};
use crate::core::engine::{AnimRequest, engine};
use crate::core::metrics::update_rem_size_from_window;
use crate::core::scheduler::AnimScheduler;
use crate::core::state::AnimState;
use crate::transition::curves::Linear;
use crate::transition::{IntoArcTransition, Transition};

type StyleState = AnimState<StyleRefinement>;
type StateModifier = Box<dyn FnOnce(StyleState) -> StyleState + Send + Sync>;

type HoverStyleFn = Arc<dyn Fn(&bool, StyleState) -> StyleState + Send + Sync>;
type ClickStyleFn = Arc<dyn Fn(&ClickEvent, StyleState) -> StyleState + Send + Sync>;

type HoverEventFn = Arc<dyn Fn(&bool, &mut Window, &mut App) + Send + Sync>;
type ClickEventFn = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + Send + Sync>;

#[derive(IntoElement)]
pub struct AnimatedWrapper<E>
where
    E: IntoElement + StatefulInteractiveElement + ParentElement + FluentBuilder + Styled + 'static,
{
    style: StyleRefinement,
    children: Vec<AnyElement>,
    id: ElementId,
    child: E,

    hover_transition: Option<(Duration, Arc<dyn Transition>)>,
    hover_modifier: Option<HoverStyleFn>,
    hover_priority: Option<AnimPriority>,
    on_hover_cb: Option<HoverEventFn>,

    click_transition: Option<(Duration, Arc<dyn Transition>)>,
    click_modifier: Option<ClickStyleFn>,
    click_priority: Option<AnimPriority>,
    on_click_cb: Option<ClickEventFn>,
}

impl<E> AnimatedWrapper<E>
where
    E: IntoElement + StatefulInteractiveElement + ParentElement + FluentBuilder + Styled + 'static,
{
    fn with_transition(mut child: E, id: impl Into<ElementId>) -> Self {
        Self {
            style: child.style().clone(),
            children: Vec::new(),
            id: id.into(),
            child,
            hover_transition: None,
            hover_modifier: None,
            hover_priority: None,
            on_hover_cb: None,
            click_transition: None,
            click_modifier: None,
            click_priority: None,
            on_click_cb: None,
        }
    }

    fn submit_transition(
        id: ElementId,
        event: AnimEvent,
        duration: Duration,
        transition: Arc<dyn Transition>,
        priority: AnimPriority,
        modifier: StateModifier,
        persistent: bool,
        initial_style: StyleRefinement,
    ) {
        engine().submit(AnimRequest {
            id,
            event,
            duration,
            transition,
            priority,
            modifier,
            persistent,
            initial_style,
        });
    }

    pub fn on_hover(
        mut self,
        callback: impl Fn(&bool, &mut Window, &mut App) + Send + Sync + 'static,
    ) -> Self {
        self.on_hover_cb = Some(Arc::new(callback));
        self
    }

    pub fn on_click(
        mut self,
        callback: impl Fn(&ClickEvent, &mut Window, &mut App) + Send + Sync + 'static,
    ) -> Self {
        self.on_click_cb = Some(Arc::new(callback));
        self
    }

    pub fn transition_on_hover<T, I>(
        mut self,
        duration: Duration,
        transition: I,
        modifier: impl Fn(&bool, StyleState) -> StyleState + Send + Sync + 'static,
    ) -> Self
    where
        T: Transition + 'static,
        I: IntoArcTransition<T>,
    {
        self.hover_transition = Some((duration, transition.into_arc()));
        self.hover_modifier = Some(Arc::new(modifier));
        self.hover_priority = Some(AnimPriority::Medium);
        self
    }

    pub fn transition_on_hover_with_priority<T, I>(
        mut self,
        duration: Duration,
        transition: I,
        priority: AnimPriority,
        modifier: impl Fn(&bool, StyleState) -> StyleState + Send + Sync + 'static,
    ) -> Self
    where
        T: Transition + 'static,
        I: IntoArcTransition<T>,
    {
        self.hover_transition = Some((duration, transition.into_arc()));
        self.hover_modifier = Some(Arc::new(modifier));
        self.hover_priority = Some(priority);
        self
    }

    pub fn transition_on_click<T, I>(
        mut self,
        duration: Duration,
        transition: I,
        modifier: impl Fn(&ClickEvent, StyleState) -> StyleState + Send + Sync + 'static,
    ) -> Self
    where
        T: Transition + 'static,
        I: IntoArcTransition<T>,
    {
        self.click_transition = Some((duration, transition.into_arc()));
        self.click_modifier = Some(Arc::new(modifier));
        self.click_priority = Some(AnimPriority::High);
        self
    }

    pub fn transition_on_click_with_priority<T, I>(
        mut self,
        duration: Duration,
        transition: I,
        priority: AnimPriority,
        modifier: impl Fn(&ClickEvent, StyleState) -> StyleState + Send + Sync + 'static,
    ) -> Self
    where
        T: Transition + 'static,
        I: IntoArcTransition<T>,
    {
        self.click_transition = Some((duration, transition.into_arc()));
        self.click_modifier = Some(Arc::new(modifier));
        self.click_priority = Some(priority);
        self
    }

    /// Declarative transition based on a condition.
    ///
    /// Note: This runs when the element is built. If the condition changes,
    /// a refresh is required to re-trigger the transition.
    pub fn transition_when<T, I>(
        self,
        condition: bool,
        duration: Duration,
        transition: I,
        modifier: impl FnOnce(StyleState) -> StyleState + Send + Sync + 'static,
    ) -> Self
    where
        T: Transition + 'static,
        I: IntoArcTransition<T>,
    {
        if condition {
            Self::submit_transition(
                self.id.clone(),
                AnimEvent::None,
                duration,
                transition.into_arc(),
                AnimPriority::Lowest,
                Box::new(modifier),
                false,
                self.style.clone(),
            );
        }

        self
    }

    pub fn transition_when_with_priority<T, I>(
        self,
        condition: bool,
        duration: Duration,
        transition: I,
        priority: AnimPriority,
        modifier: impl FnOnce(StyleState) -> StyleState + Send + Sync + 'static,
    ) -> Self
    where
        T: Transition + 'static,
        I: IntoArcTransition<T>,
    {
        if condition {
            Self::submit_transition(
                self.id.clone(),
                AnimEvent::None,
                duration,
                transition.into_arc(),
                priority,
                Box::new(modifier),
                false,
                self.style.clone(),
            );
        }

        self
    }

    pub fn transition_when_else<T, I>(
        self,
        condition: bool,
        duration: Duration,
        transition: I,
        then: impl FnOnce(StyleState) -> StyleState + Send + Sync + 'static,
        else_fn: impl FnOnce(StyleState) -> StyleState + Send + Sync + 'static,
    ) -> Self
    where
        T: Transition + 'static,
        I: IntoArcTransition<T>,
    {
        if condition {
            Self::submit_transition(
                self.id.clone(),
                AnimEvent::None,
                duration,
                transition.into_arc(),
                AnimPriority::Lowest,
                Box::new(then),
                false,
                self.style.clone(),
            );
        } else {
            Self::submit_transition(
                self.id.clone(),
                AnimEvent::None,
                duration,
                transition.into_arc(),
                AnimPriority::Lowest,
                Box::new(else_fn),
                false,
                self.style.clone(),
            );
        }

        self
    }

    pub fn transition_when_else_with_priority<T, I>(
        self,
        condition: bool,
        duration: Duration,
        transition: I,
        priority: AnimPriority,
        then: impl FnOnce(StyleState) -> StyleState + Send + Sync + 'static,
        else_fn: impl FnOnce(StyleState) -> StyleState + Send + Sync + 'static,
    ) -> Self
    where
        T: Transition + 'static,
        I: IntoArcTransition<T>,
    {
        if condition {
            Self::submit_transition(
                self.id.clone(),
                AnimEvent::None,
                duration,
                transition.into_arc(),
                priority,
                Box::new(then),
                false,
                self.style.clone(),
            );
        } else {
            Self::submit_transition(
                self.id.clone(),
                AnimEvent::None,
                duration,
                transition.into_arc(),
                priority,
                Box::new(else_fn),
                false,
                self.style.clone(),
            );
        }

        self
    }

    pub fn transition_when_some<T, I, O>(
        self,
        option: Option<O>,
        duration: Duration,
        transition: I,
        modifier: impl FnOnce(StyleState) -> StyleState + Send + Sync + 'static,
    ) -> Self
    where
        T: Transition + 'static,
        I: IntoArcTransition<T>,
    {
        if option.is_some() {
            Self::submit_transition(
                self.id.clone(),
                AnimEvent::None,
                duration,
                transition.into_arc(),
                AnimPriority::Lowest,
                Box::new(modifier),
                false,
                self.style.clone(),
            );
        }

        self
    }

    pub fn transition_when_some_with_priority<T, I, O>(
        self,
        option: Option<O>,
        duration: Duration,
        transition: I,
        priority: AnimPriority,
        modifier: impl FnOnce(StyleState) -> StyleState + Send + Sync + 'static,
    ) -> Self
    where
        T: Transition + 'static,
        I: IntoArcTransition<T>,
    {
        if option.is_some() {
            Self::submit_transition(
                self.id.clone(),
                AnimEvent::None,
                duration,
                transition.into_arc(),
                priority,
                Box::new(modifier),
                false,
                self.style.clone(),
            );
        }

        self
    }

    pub fn transition_when_none<T, I, O>(
        self,
        option: &Option<O>,
        duration: Duration,
        transition: I,
        modifier: impl FnOnce(StyleState) -> StyleState + Send + Sync + 'static,
    ) -> Self
    where
        T: Transition + 'static,
        I: IntoArcTransition<T>,
    {
        if option.is_none() {
            Self::submit_transition(
                self.id.clone(),
                AnimEvent::None,
                duration,
                transition.into_arc(),
                AnimPriority::Lowest,
                Box::new(modifier),
                false,
                self.style.clone(),
            );
        }

        self
    }

    pub fn transition_when_none_with_priority<T, I, O>(
        self,
        option: &Option<O>,
        duration: Duration,
        transition: I,
        priority: AnimPriority,
        modifier: impl FnOnce(StyleState) -> StyleState + Send + Sync + 'static,
    ) -> Self
    where
        T: Transition + 'static,
        I: IntoArcTransition<T>,
    {
        if option.is_none() {
            Self::submit_transition(
                self.id.clone(),
                AnimEvent::None,
                duration,
                transition.into_arc(),
                priority,
                Box::new(modifier),
                false,
                self.style.clone(),
            );
        }

        self
    }
}

impl<E> Styled for AnimatedWrapper<E>
where
    E: IntoElement + StatefulInteractiveElement + ParentElement + FluentBuilder + Styled + 'static,
{
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl<E> ParentElement for AnimatedWrapper<E>
where
    E: IntoElement + StatefulInteractiveElement + ParentElement + FluentBuilder + Styled + 'static,
{
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl<E> RenderOnce for AnimatedWrapper<E>
where
    E: IntoElement + StatefulInteractiveElement + ParentElement + FluentBuilder + Styled + 'static,
{
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        update_rem_size_from_window(window);
        AnimScheduler::init(cx);

        // Cache initial style before it's consumed
        let initial_style = self.style.clone();

        let mut root = self.child;
        root.style().refine(&self.style);

        if let Some(state) = engine().state(&self.id) {
            root.style().refine(&state.cur);
        }

        let id_for_hover = self.id.clone();
        let hover_transition = self
            .hover_transition
            .unwrap_or_else(|| (Duration::default(), Arc::new(Linear)));
        let hover_modifier = self.hover_modifier;
        let hover_priority = self.hover_priority.unwrap_or(AnimPriority::Medium);
        let on_hover_cb = self.on_hover_cb;

        let id_for_click = self.id.clone();
        let click_transition = self
            .click_transition
            .unwrap_or_else(|| (Duration::default(), Arc::new(Linear)));
        let click_modifier = self.click_modifier;
        let click_priority = self.click_priority.unwrap_or(AnimPriority::High);
        let on_click_cb = self.on_click_cb;

        let initial_for_hover = initial_style.clone();
        let initial_for_click = initial_style.clone();

        root.on_hover(move |hovered, window, app| {
            if let Some(cb) = on_hover_cb.as_ref() {
                cb(hovered, window, app);
            }

            if let Some(modifier) = hover_modifier.clone() {
                let hovered_value = *hovered;
                let transition = hover_transition.clone();
                Self::submit_transition(
                    id_for_hover.clone(),
                    AnimEvent::Hover,
                    transition.0,
                    transition.1,
                    hover_priority,
                    Box::new(move |state| {
                        let hovered_local = hovered_value;
                        (modifier)(&hovered_local, state)
                    }),
                    hovered_value,
                    initial_for_hover.clone(),
                );
            }
        })
        .on_click(move |event, window, app| {
            if let Some(cb) = on_click_cb.as_ref() {
                cb(event, window, app);
            }

            if let Some(modifier) = click_modifier.clone() {
                let transition = click_transition.clone();
                let event_cloned = event.clone();
                Self::submit_transition(
                    id_for_click.clone(),
                    AnimEvent::Click,
                    transition.0,
                    transition.1,
                    click_priority,
                    Box::new(move |state| (modifier)(&event_cloned, state)),
                    false,
                    initial_for_click.clone(),
                );
            }
        })
        .children(self.children)
    }
}

pub trait TransitionExt:
    IntoElement + StatefulInteractiveElement + ParentElement + FluentBuilder + Styled + 'static
{
    fn with_transition(self, id: impl Into<ElementId>) -> AnimatedWrapper<Self> {
        AnimatedWrapper::with_transition(self, id)
    }
}

impl<T> TransitionExt for T where
    T: IntoElement + StatefulInteractiveElement + ParentElement + FluentBuilder + Styled + 'static
{
}
