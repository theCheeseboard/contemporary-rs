use crate::components::admonitions::Admonitions;
use crate::components::buttons::Buttons;
use crate::components::checkboxes_radio_buttons::CheckboxesRadioButtons;
use crate::components::interstitials::Interstitials;
use crate::components::progress_bars::ProgressBars;
use crate::components::ranges::Ranges;
use crate::components::scroll_areas::ScrollAreas;
use crate::components::skeletons::Skeletons;
use crate::components::text_input::TextInput;
use cntp_i18n::{I18nString, tr};
use contemporary::components::grandstand::grandstand;
use contemporary::components::layer::layer;
use contemporary::components::pager::lift_animation::LiftAnimation;
use contemporary::components::pager::pager_animation::PagerAnimationDirection;
use contemporary::components::pager::{ManagedPagerPage, PageNumber, pager_managed};
use contemporary::components::scrollbar::SelfScrollable;
use contemporary::styling::theme::ThemeStorage;
use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement,
    Render, RenderOnce, StatefulInteractiveElement, Styled, Window, div, px, uniform_list,
};

pub struct ComponentsRoot {
    buttons: Entity<Buttons>,
    checkboxes_radio_buttons: Entity<CheckboxesRadioButtons>,
    text_input: Entity<TextInput>,
    progress_bars: Entity<ProgressBars>,
    ranges: Entity<Ranges>,
    skeletons: Entity<Skeletons>,
    admonitions: Entity<Admonitions>,
    interstitials: Entity<Interstitials>,
    scroll_areas: Entity<ScrollAreas>,

    current_page: ComponentsPage,
}

#[derive(Clone)]
pub enum ComponentsPage {
    Buttons(Entity<Buttons>),
    CheckboxesRadioButtons(Entity<CheckboxesRadioButtons>),
    TextInput(Entity<TextInput>),
    ProgressBars(Entity<ProgressBars>),
    Ranges(Entity<Ranges>),
    Skeletons(Entity<Skeletons>),
    Admonitions(Entity<Admonitions>),
    Interstitials(Entity<Interstitials>),
    ScrollAreas(Entity<ScrollAreas>),
}

impl ComponentsPage {
    fn name(&self) -> I18nString {
        match self {
            ComponentsPage::Buttons(_) => {
                tr!("BUTTONS_TITLE")
            }
            ComponentsPage::CheckboxesRadioButtons(_) => {
                tr!("CHECKBOXES_RADIO_BUTTONS_TITLE")
            }
            ComponentsPage::TextInput(_) => {
                tr!("TEXT_INPUT_TITLE")
            }
            ComponentsPage::ProgressBars(_) => {
                tr!("PROGRESS_BARS_TITLE")
            }
            ComponentsPage::Ranges(_) => {
                tr!("RANGES_TITLE")
            }
            ComponentsPage::Skeletons(_) => {
                tr!("SKELETONS_TITLE")
            }
            ComponentsPage::Admonitions(_) => {
                tr!("ADMONITIONS_TITLE")
            }
            ComponentsPage::Interstitials(_) => {
                tr!("INTERSTITIALS_TITLE", "Interstitials")
            }
            ComponentsPage::ScrollAreas(_) => {
                tr!("SCROLL_AREAS_TITLE", "Scroll Areas")
            }
        }
    }
}

impl PageNumber for ComponentsPage {
    fn page_number(&self) -> usize {
        match self {
            ComponentsPage::Buttons(_) => 0,
            ComponentsPage::CheckboxesRadioButtons(_) => 1,
            ComponentsPage::TextInput(_) => 2,
            ComponentsPage::ProgressBars(_) => 3,
            ComponentsPage::Ranges(_) => 4,
            ComponentsPage::Skeletons(_) => 5,
            ComponentsPage::Admonitions(_) => 6,
            ComponentsPage::Interstitials(_) => 7,
            ComponentsPage::ScrollAreas(_) => 8,
        }
    }
}

impl ManagedPagerPage for ComponentsPage {
    fn render(&self, window: &mut Window, cx: &mut App) -> AnyElement {
        <Self as RenderOnce>::render(self.clone(), window, cx).into_any_element()
    }
}

impl RenderOnce for ComponentsPage {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        match self {
            ComponentsPage::Buttons(buttons) => buttons.clone().into_any_element(),
            ComponentsPage::CheckboxesRadioButtons(checkboxes_radio_buttons) => {
                checkboxes_radio_buttons.into_any_element()
            }
            ComponentsPage::TextInput(text_input) => text_input.clone().into_any_element(),
            ComponentsPage::ProgressBars(progress_bars) => progress_bars.clone().into_any_element(),
            ComponentsPage::Ranges(ranges) => ranges.clone().into_any_element(),
            ComponentsPage::Skeletons(skeletons) => skeletons.clone().into_any_element(),
            ComponentsPage::Admonitions(admonitions) => admonitions.clone().into_any_element(),
            ComponentsPage::Interstitials(interstitials) => {
                interstitials.clone().into_any_element()
            }
            ComponentsPage::ScrollAreas(scroll_areas) => scroll_areas.clone().into_any_element(),
        }
    }
}

impl ComponentsRoot {
    pub fn new(cx: &mut App) -> Entity<ComponentsRoot> {
        cx.new(|cx| {
            let buttons = Buttons::new(cx);
            ComponentsRoot {
                buttons: buttons.clone(),
                checkboxes_radio_buttons: CheckboxesRadioButtons::new(cx),
                text_input: TextInput::new(cx),
                progress_bars: ProgressBars::new(cx),
                ranges: Ranges::new(cx),
                skeletons: Skeletons::new(cx),
                admonitions: Admonitions::new(cx),
                interstitials: Interstitials::new(cx),
                scroll_areas: cx.new(ScrollAreas::new),
                current_page: ComponentsPage::Buttons(buttons),
            }
        })
    }
}

impl Render for ComponentsRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pages = vec![
            ComponentsPage::Buttons(self.buttons.clone()),
            ComponentsPage::CheckboxesRadioButtons(self.checkboxes_radio_buttons.clone()),
            ComponentsPage::TextInput(self.text_input.clone()),
            ComponentsPage::ProgressBars(self.progress_bars.clone()),
            ComponentsPage::Ranges(self.ranges.clone()),
            ComponentsPage::Skeletons(self.skeletons.clone()),
            ComponentsPage::Admonitions(self.admonitions.clone()),
            ComponentsPage::Interstitials(self.interstitials.clone()),
            ComponentsPage::ScrollAreas(self.scroll_areas.clone()),
        ];

        div()
            .id("components")
            .flex()
            .w_full()
            .h_full()
            .gap(px(2.))
            .child(
                layer()
                    .w(px(300.))
                    .flex()
                    .flex_col()
                    .child(
                        grandstand("sidebar-grandstand")
                            .text(tr!("COMPONENTS_TITLE", "Components"))
                            .pt(px(36.)),
                    )
                    .child(
                        div().flex_grow().p(px(2.)).child(
                            uniform_list(
                                "sidebar-items",
                                9,
                                cx.processor(move |this, range, _, cx| {
                                    let theme = cx.theme();
                                    let mut items = Vec::new();

                                    let pages: &[ComponentsPage] = &pages[range];
                                    for page in pages {
                                        items.push(
                                            div()
                                                .id(page.page_number())
                                                .p(px(2.))
                                                .rounded(theme.border_radius)
                                                .on_click({
                                                    let page = page.clone();
                                                    cx.listener(move |this, _, _, cx| {
                                                        this.current_page = page.clone();
                                                        cx.notify()
                                                    })
                                                })
                                                .child(page.name())
                                                .when(
                                                    this.current_page.page_number()
                                                        == page.page_number(),
                                                    |div| div.bg(theme.button_background),
                                                ),
                                        );
                                    }
                                    items
                                }),
                            )
                            .self_scrollable(window, cx)
                            .h_full()
                            .w_full(),
                        ),
                    ),
            )
            .child(
                pager_managed("main-area", self.current_page.clone())
                    .flex_grow()
                    .animation(LiftAnimation::new())
                    .animation_direction(PagerAnimationDirection::Forward),
            )
    }
}
