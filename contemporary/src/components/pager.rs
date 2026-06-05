pub mod fade_animation;
pub mod lift_animation;
pub mod pager_animation;
pub mod slide_horizontal_animation;

use crate::components::pager::pager_animation::{
    PagerAnimation, PagerAnimationDirection, PagerElement,
};
use crate::platform_support::platform_settings::PlatformSettings;
use gpui::{
    AnyElement, App, Bounds, Div, Element, ElementId, GlobalElementId, InspectorElementId,
    InteractiveElement, IntoElement, LayoutId, ParentElement, Pixels, Refineable, RenderOnce,
    Style, StyleRefinement, Styled, Window, div,
};
use std::collections::HashMap;
use std::panic::Location;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(IntoElement)]
pub struct Pager {
    id: ElementId,
    elements: Vec<Option<Div>>,
    page: usize,
    style_refinement: StyleRefinement,
    animation: Option<Box<dyn PagerAnimation>>,
    force_direction: Option<PagerAnimationDirection>,
}

pub trait PageNumber {
    fn page_number(&self) -> usize;
}

impl PageNumber for usize {
    fn page_number(&self) -> usize {
        *self
    }
}

impl<T> PageNumber for Box<T>
where
    T: PageNumber,
{
    fn page_number(&self) -> usize {
        self.as_ref().page_number()
    }
}

impl<T> PageNumber for Rc<T>
where
    T: PageNumber,
{
    fn page_number(&self) -> usize {
        self.as_ref().page_number()
    }
}

impl<T> PageNumber for Arc<T>
where
    T: PageNumber,
{
    fn page_number(&self) -> usize {
        self.as_ref().page_number()
    }
}

pub fn pager(id: impl Into<ElementId>, page: impl PageNumber) -> Pager {
    Pager {
        id: id.into(),
        elements: vec![],
        page: page.page_number(),
        style_refinement: StyleRefinement::default(),
        animation: None,
        force_direction: None,
    }
}

impl Pager {
    pub fn page(mut self, element: impl IntoElement) -> Pager {
        self.elements.push(Some(
            div().absolute().w_full().h_full().occlude().child(element),
        ));
        self
    }

    pub fn animation(mut self, animation: Box<dyn PagerAnimation>) -> Self {
        self.animation = Some(animation);
        self
    }

    pub fn animation_direction(mut self, direction: PagerAnimationDirection) -> Self {
        self.force_direction = Some(direction);
        self
    }
}

impl Styled for Pager {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style_refinement
    }
}

impl RenderOnce for Pager {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let platform_settings = cx.global::<PlatformSettings>();
        PagerInternal {
            id: self.id,
            style_refinement: self.style_refinement,
            current_page_number: self.page,
            elements: self.elements,
            animation_duration: platform_settings.animation_duration,
            animation: self.animation,
            force_direction: self.force_direction,
        }
    }
}

struct PagerInternal {
    id: ElementId,
    current_page_number: usize,
    elements: Vec<Option<Div>>,
    style_refinement: StyleRefinement,
    animation_duration: Duration,
    animation: Option<Box<dyn PagerAnimation>>,
    force_direction: Option<PagerAnimationDirection>,
}

impl IntoElement for PagerInternal {
    type Element = PagerInternal;

    fn into_element(self) -> Self::Element {
        self
    }
}

struct PagerInternalState {
    animation_start: Instant,
    previous_current_page: usize,
    current_page: usize,
}

struct PagerRequestLayoutState {
    current_page: AnyElement,
    previous_page: AnyElement,
    animation_done: bool,
    top_element: PagerElement,
}

impl Element for PagerInternal {
    type RequestLayoutState = PagerRequestLayoutState;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        window.with_element_state(id.unwrap(), |state, window| {
            let mut state = state.unwrap_or_else(|| PagerInternalState {
                animation_start: Instant::now(),
                previous_current_page: self.current_page_number,
                current_page: self.current_page_number,
            });

            if state.current_page != self.current_page_number {
                state.animation_start = Instant::now();
                state.previous_current_page = state.current_page;
                state.current_page = self.current_page_number;
            }

            let mut delta = state.animation_start.elapsed().as_secs_f32()
                / self.animation_duration.as_secs_f32();

            let mut done = false;
            if delta > 1.0 {
                done = true;
                delta = 1.0;
            }

            let mut current_page = self
                .elements
                .get_mut(state.current_page)
                .unwrap_or(&mut None)
                .take()
                .unwrap_or_else(|| div());
            let mut previous_page = self
                .elements
                .get_mut(state.previous_current_page)
                .unwrap_or(&mut None)
                .take()
                .unwrap_or_else(|| div());

            let mut top_element = PagerElement::Current;

            // Set up animations
            if let Some(animation) = &mut self.animation {
                let animation_direction = self.force_direction.unwrap_or({
                    if state.current_page > state.previous_current_page {
                        PagerAnimationDirection::Forward
                    } else {
                        PagerAnimationDirection::Backward
                    }
                });
                previous_page = animation.animate_out(previous_page, animation_direction, delta);
                current_page = animation.animate_in(current_page, animation_direction, delta);
                top_element = animation.top_element(animation_direction);
            }

            let mut previous_page = previous_page.into_any_element();
            let mut current_page = current_page.into_any_element();

            let previous_page_layout_id = previous_page.request_layout(window, cx);
            let current_page_layout_id = current_page.request_layout(window, cx);

            let final_layout_id = window.request_layout(
                Style::default().refined(self.style_refinement.clone()),
                vec![current_page_layout_id, previous_page_layout_id],
                cx,
            );

            if !done {
                window.request_animation_frame();
            }

            (
                (
                    final_layout_id,
                    PagerRequestLayoutState {
                        current_page,
                        previous_page,
                        animation_done: done,
                        top_element,
                    },
                ),
                state,
            )
        })
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        if !request_layout.animation_done && self.animation.is_some() {
            request_layout.previous_page.prepaint(window, cx);
        }
        request_layout.current_page.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        if request_layout.top_element == PagerElement::Previous {
            // Render the current page at the bottom
            request_layout.current_page.paint(window, cx);
        }
        if !request_layout.animation_done && self.animation.is_some() {
            request_layout.previous_page.paint(window, cx);
        }
        if request_layout.top_element == PagerElement::Current {
            // Render the current page at the top
            request_layout.current_page.paint(window, cx);
        }
    }
}

#[derive(IntoElement)]
pub struct ManagedPager {
    id: ElementId,
    page: Rc<dyn ManagedPagerPage>,
    style_refinement: StyleRefinement,
    animation: Option<Box<dyn PagerAnimation>>,
    force_direction: Option<PagerAnimationDirection>,
}

pub trait ManagedPagerPage: PageNumber {
    fn render(&self, window: &mut Window, cx: &mut App) -> AnyElement;
}

pub fn pager_managed(
    id: impl Into<ElementId>,
    page: impl ManagedPagerPage + 'static,
) -> ManagedPager {
    ManagedPager {
        id: id.into(),
        page: Rc::new(page),
        style_refinement: StyleRefinement::default(),
        animation: None,
        force_direction: None,
    }
}

#[derive(Default)]
struct ManagedPagerState {
    cached_pages: HashMap<usize, Rc<dyn ManagedPagerPage>>,
}

impl ManagedPager {
    pub fn animation(mut self, animation: Box<dyn PagerAnimation>) -> Self {
        self.animation = Some(animation);
        self
    }

    pub fn animation_direction(mut self, direction: PagerAnimationDirection) -> Self {
        self.force_direction = Some(direction);
        self
    }
}

impl Styled for ManagedPager {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style_refinement
    }
}

impl RenderOnce for ManagedPager {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let page = self.page;
        let pager_state = window.use_state(cx, |_, _| ManagedPagerState::default());

        let max_page = pager_state.update(cx, |state, cx| {
            state.cached_pages.insert(page.page_number(), page.clone());

            *state.cached_pages.keys().max().unwrap()
        });

        let mut pager = pager(self.id, page.page_number());
        for i in 0..=max_page {
            pager = pager.page(
                pager_state
                    .read(cx)
                    .cached_pages
                    .get(&i)
                    .cloned()
                    .map(|x| x.render(window, cx))
                    .unwrap_or_else(|| div().into_any_element()),
            );
        }
        if let Some(animation) = self.animation {
            pager = pager.animation(animation)
        }
        if let Some(direction) = self.force_direction {
            pager = pager.animation_direction(direction);
        }
        pager.style().refine(&self.style_refinement);
        pager
    }
}
