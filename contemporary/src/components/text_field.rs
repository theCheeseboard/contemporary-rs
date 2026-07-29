use crate::components::context_menu::{ContextMenuExt, ContextMenuItem};
use crate::components::error_flasher::ErrorFlasher;
use crate::components::focus_decoration::focus_decoration;
use crate::styling::theme::{ThemeStorage, VariableColor};
use cntp_i18n::tr;
use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, Bounds, ClipboardItem, Context, Element, ElementId, ElementInputHandler,
    Entity, EntityInputHandler, FocusHandle, GlobalElementId, Hsla, InspectorElementId,
    InteractiveElement, IntoElement, KeyBinding, LayoutId, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, PaintQuad, ParentElement, Pixels, Point, Render, Rgba,
    ShapedLine, Style, Styled, TextAlign, TextRun, TextStyleRefinement, UTF16Selection,
    UnderlineStyle, Window, actions, div, fill, point, px, relative, size,
};
use std::ops::Range;
use std::panic::Location;
use std::rc::Rc;
use unicode_segmentation::UnicodeSegmentation;

actions!(
    text_field,
    [
        Enter,
        NewLine,
        Backspace,
        Delete,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        ShowCharacterPalette,
        Paste,
        Cut,
        Copy,
        Quit,
    ]
);

pub fn bind_text_field_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("enter", Enter, None),
        KeyBinding::new("shift-enter", NewLine, None),
        KeyBinding::new("backspace", Backspace, None),
        KeyBinding::new("delete", Delete, None),
        KeyBinding::new("left", Left, None),
        KeyBinding::new("right", Right, None),
        KeyBinding::new("shift-left", SelectLeft, None),
        KeyBinding::new("shift-right", SelectRight, None),
        KeyBinding::new("secondary-a", SelectAll, None),
        KeyBinding::new("secondary-v", Paste, None),
        KeyBinding::new("secondary-c", Copy, None),
        KeyBinding::new("secondary-x", Cut, None),
        KeyBinding::new("home", Home, None),
        KeyBinding::new("end", End, None),
        KeyBinding::new("ctrl-cmd-space", ShowCharacterPalette, None),
    ]);
}

pub type EnterPressListener = dyn Fn(&EnterPressEvent, &mut Window, &mut App) + 'static;
pub type TextChangedListener = dyn Fn(&TextChangedEvent, &mut Window, &mut App) + 'static;
pub type PasteRichListener = dyn Fn(&PasteRichEvent, &mut Window, &mut App) + 'static;

#[derive(Clone)]
pub struct EnterPressEvent;
#[derive(Clone)]
pub struct TextChangedEvent;
#[derive(Clone)]
pub struct PasteRichEvent {
    pub clipboard_item: ClipboardItem,
}

#[derive(Clone, Default)]
pub enum MaskMode {
    #[default]
    Clear,
    Mask(char),
}

impl MaskMode {
    pub fn password_mask() -> Self {
        // TODO: We stumble when the number of bytes in the character doesn't match
        // MaskMode::Mask('●')
        MaskMode::Mask('*')
    }
}

pub struct TextField {
    id: ElementId,
    text: String,
    placeholder: String,
    focus_handle: FocusHandle,
    marked_range: Option<Range<usize>>,
    selected_range: Range<usize>,
    selection_reversed: bool,
    last_layout: Option<Vec<ShapedLine>>,
    last_bounds: Option<Bounds<Pixels>>,
    is_selecting: bool,
    has_border: bool,
    allow_new_lines: bool,
    mask_mode: MaskMode,

    text_style: TextStyleRefinement,
    error_flasher: Entity<ErrorFlasher>,

    enter_press_listener: Option<Rc<Box<EnterPressListener>>>,
    text_changed_listener: Option<Rc<Box<TextChangedListener>>>,
    paste_rich_listener: Option<Rc<Box<PasteRichListener>>>,
}

impl TextField {
    pub fn new(id: impl Into<ElementId>, cx: &mut Context<Self>) -> Self {
        let error_flasher = cx.new(ErrorFlasher::new);

        cx.observe(&error_flasher, |_, _, cx| {
            cx.notify();
        })
        .detach();

        Self {
            id: id.into(),
            text: "".into(),
            placeholder: "".into(),
            focus_handle: cx.focus_handle(),
            marked_range: None,
            selected_range: 0..0,
            selection_reversed: false,
            last_layout: None,
            last_bounds: None,
            is_selecting: false,
            has_border: true,
            allow_new_lines: false,
            mask_mode: MaskMode::Clear,
            text_style: TextStyleRefinement::default(),
            error_flasher,
            enter_press_listener: None,
            text_changed_listener: None,
            paste_rich_listener: None,
        }
    }

    pub fn set_placeholder(&mut self, placeholder: &str) {
        self.placeholder = placeholder.into();
    }

    pub fn set_text(&mut self, text: &str) {
        self.reset();
        self.text = text.into();
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_has_border(&mut self, has_border: bool) {
        self.has_border = has_border;
    }

    pub fn has_border(&self) -> bool {
        self.has_border
    }

    pub fn set_mask_mode(&mut self, mask_mode: MaskMode) {
        self.mask_mode = mask_mode;
    }

    pub fn mask_mode(&self) -> &MaskMode {
        &self.mask_mode
    }

    pub fn reset(&mut self) {
        self.text = "".into();
        self.selected_range = 0..0;
        self.selection_reversed = false;
        self.marked_range = None;
        self.last_layout = None;
        self.last_bounds = None;
        self.is_selecting = false;
    }

    pub fn type_string(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.replace_text_in_range(None, text, window, cx);
    }

    pub fn on_enter_press(
        &mut self,
        listener: impl Fn(&EnterPressEvent, &mut Window, &mut App) + 'static,
    ) {
        self.enter_press_listener = Some(Rc::new(Box::new(listener)));
    }

    pub fn on_text_changed(
        &mut self,
        listener: impl Fn(&TextChangedEvent, &mut Window, &mut App) + 'static,
    ) {
        self.text_changed_listener = Some(Rc::new(Box::new(listener)));
    }

    pub fn on_paste_rich(
        &mut self,
        listener: impl Fn(&PasteRichEvent, &mut Window, &mut App) + 'static,
    ) {
        self.paste_rich_listener = Some(Rc::new(Box::new(listener)));
    }

    pub fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;

        for ch in self.text.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }

        utf8_offset
    }

    pub fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;

        for ch in self.text.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }

        utf16_offset
    }

    pub fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    pub fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    pub fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.is_selecting = true;

        if event.modifiers.shift {
            self.select_to(self.index_for_mouse_position(window, event.position), cx);
        } else {
            self.move_to(self.index_for_mouse_position(window, event.position), cx)
        }
    }

    pub fn on_mouse_up(&mut self, _: &MouseUpEvent, _window: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }

    pub fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_selecting {
            self.select_to(self.index_for_mouse_position(window, event.position), cx);
        }
    }

    pub fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        cx.notify()
    }

    pub fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    pub fn index_for_mouse_position(&self, window: &mut Window, position: Point<Pixels>) -> usize {
        if self.text.is_empty() {
            return 0;
        }

        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };
        if position.y < bounds.top() {
            return 0;
        }
        if position.y > bounds.bottom() {
            return self.text.len();
        }
        let line_height = window.line_height();
        let line_index = ((position.y - bounds.top()) / line_height).floor() as usize;
        line[line_index].closest_index_for_x(position.x - bounds.left())
            // Add on the line separators
            + line_index
            // And the length of the previous lines
            + split_string_lines(&self.text)
            .take(line_index)
            .map(|line| line.len())
            .sum::<usize>()
    }

    pub fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset
        } else {
            self.selected_range.end = offset
        };
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        cx.notify()
    }

    pub fn previous_boundary(&self, offset: usize) -> usize {
        self.text
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    pub fn next_boundary(&self, offset: usize) -> usize {
        self.text
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.text.len())
    }

    pub fn enter(&mut self, _: &Enter, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(enter_press_listener) = &self.enter_press_listener {
            enter_press_listener(&EnterPressEvent, window, cx);
        }
    }

    pub fn new_line(&mut self, _: &NewLine, window: &mut Window, cx: &mut Context<Self>) {
        if self.allow_new_lines {
            self.replace_text_in_range(None, "\n", window, cx);
        }
    }

    pub fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.previous_boundary(self.cursor_offset()), cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    pub fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.next_boundary(self.cursor_offset()), cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    pub fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.previous_boundary(self.cursor_offset()), cx);
        } else {
            self.move_to(self.selected_range.start, cx)
        }
    }

    pub fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.next_boundary(self.selected_range.end), cx);
        } else {
            self.move_to(self.selected_range.end, cx)
        }
    }

    pub fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()), cx);
    }

    pub fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.next_boundary(self.cursor_offset()), cx);
    }

    pub fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
        self.select_to(self.text.len(), cx)
    }

    pub fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
    }

    pub fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.text.len(), cx);
    }

    pub fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(clipboard_item) = cx.read_from_clipboard() {
            if let Some(text) = clipboard_item.text() {
                self.replace_text_in_range(None, &text, window, cx);
            } else if let Some(paste_rich_event) = self.paste_rich_listener.as_ref() {
                paste_rich_event(&PasteRichEvent { clipboard_item }, window, cx);
            }
        }
    }

    pub fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.text[self.selected_range.clone()].to_string(),
            ));
        }
    }

    pub fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.text[self.selected_range.clone()].to_string(),
            ));
            self.replace_text_in_range(None, "", window, cx)
        }
    }

    pub fn text_style(&mut self) -> &mut TextStyleRefinement {
        &mut self.text_style
    }

    pub fn flash_error(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.error_flasher
            .update(cx, |error_flasher, cx| error_flasher.flash(window, cx))
    }
}

impl Render for TextField {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let error_flasher_color = self.error_flasher.read(cx).color();
        div()
            .id(self.id.clone())
            .track_focus(&self.focus_handle)
            .flex()
            .w_full()
            .cursor_text()
            .key_context("ChatInput")
            .on_action(cx.listener(Self::enter))
            .on_action(cx.listener(Self::new_line))
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            // .on_action(cx.listener(Self::show_character_palette))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .when(
                self.focus_handle.contains_focused(window, cx) && self.has_border,
                |david| david.child(focus_decoration().absolute().top_0().left_0().size_full()),
            )
            .child(
                div()
                    .flex_grow(1.)
                    .when(self.has_border, |david| {
                        david
                            .p(px(2.))
                            .rounded(theme.border_radius)
                            .border(px(1.))
                            .border_color(theme.border_color)
                            .bg(theme.layer_background)
                    })
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .top_0()
                            .size_full()
                            .rounded(theme.border_radius)
                            .bg(error_flasher_color),
                    )
                    .child(
                        div()
                            .w_full()
                            .p(px(4.))
                            .child(TextFieldElement { input: cx.entity() }),
                    ),
            )
            .with_context_menu(vec![
                ContextMenuItem::menu_item()
                    .icon("edit-cut")
                    .label(tr!("EDIT_CUT", "Cut"))
                    .on_triggered(cx.listener(|this, _, window, cx| this.cut(&Cut, window, cx)))
                    .build(),
                ContextMenuItem::menu_item()
                    .icon("edit-copy")
                    .label(tr!("EDIT_COPY", "Copy"))
                    .on_triggered(cx.listener(|this, _, window, cx| this.copy(&Copy, window, cx)))
                    .build(),
                ContextMenuItem::menu_item()
                    .icon("edit-paste")
                    .label(tr!("EDIT_PASTE", "Paste"))
                    .on_triggered(cx.listener(|this, _, window, cx| this.paste(&Paste, window, cx)))
                    .build(),
            ])
    }
}

impl EntityInputHandler for TextField {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.text[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.text = self.text[0..range.start].to_owned() + new_text + &self.text[range.end..];
        self.selected_range = range.start + new_text.len()..range.start + new_text.len();
        self.marked_range.take();

        if let Some(text_changed_listener) = &self.text_changed_listener {
            text_changed_listener(&TextChangedEvent, window, cx);
        }

        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.text = self.text[0..range.start].to_owned() + new_text + &self.text[range.end..];
        if !new_text.is_empty() {
            self.marked_range = Some(range.start..range.start + new_text.len());
        } else {
            self.marked_range = None;
        }
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .map(|new_range| new_range.start + range.start..new_range.end + range.end)
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());

        if let Some(text_changed_listener) = &self.text_changed_listener {
            text_changed_listener(&TextChangedEvent, window, cx);
        }

        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        _range_utf16: Range<usize>,
        _bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        // let last_layout = self.last_layout.as_ref()?;
        // let range = self.range_from_utf16(&range_utf16);
        // Some(Bounds::from_corners(
        //     point(
        //         bounds.left() + last_layout.x_for_index(range.start),
        //         bounds.top(),
        //     ),
        //     point(
        //         bounds.left() + last_layout.x_for_index(range.end),
        //         bounds.bottom(),
        //     ),
        // ))
        None
    }

    fn character_index_for_point(
        &mut self,
        _point: gpui::Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        // let line_point = self.last_bounds?.localize(&point)?;
        // let last_layout = self.last_layout.as_ref()?;
        //
        // assert_eq!(last_layout.text, self.text);
        // let utf8_index = last_layout.index_for_x(point.x - line_point.x)?;
        // Some(self.offset_to_utf16(utf8_index))
        None
    }
}

pub struct TextFieldElement {
    input: Entity<TextField>,
}

pub struct RequestLayoutState {
    lines: Vec<String>,
    text_color: Hsla,
}

pub struct PrepaintState {
    lines: Vec<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Vec<PaintQuad>,
}

impl IntoElement for TextFieldElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextFieldElement {
    type RequestLayoutState = RequestLayoutState;
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let input = self.input.read(cx);
        let content = input.text.clone();
        let style = window.text_style();
        let theme = cx.theme();
        let mask_mode = &input.mask_mode;

        let (display_text, text_color) = if content.is_empty() {
            (
                input.placeholder.clone(),
                Hsla::from(theme.foreground.disabled()),
            )
        } else {
            (content.clone(), style.color)
        };

        let lines = split_string_lines(&display_text)
            .map(|text| text.to_string())
            .map(|text| match mask_mode {
                // Don't mask the placeholder string
                MaskMode::Mask(mask_character) if !content.is_empty() => {
                    mask_character.to_string().as_str().repeat(text.len())
                }
                _ => text,
            })
            .collect::<Vec<_>>();

        window.with_text_style(Some(input.text_style.clone()), |window| {
            let mut style = Style::default();
            style.size.width = relative(1.).into();
            style.size.height = (window.line_height() * lines.len()).into();
            (
                window.request_layout(style, [], cx),
                RequestLayoutState { lines, text_color },
            )
        })
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        window.with_text_style(Some(input.text_style.clone()), |window| {
            let style = window.text_style();
            let cursor = input.cursor_offset();
            let selected_range = input.selected_range.clone();
            let theme = cx.theme();
            let line_height = window.line_height();

            let lines = request_layout
                .lines
                .iter()
                .map(|text| {
                    let run = TextRun {
                        len: text.len(),
                        font: style.font(),
                        color: request_layout.text_color,
                        background_color: None,
                        underline: None,
                        strikethrough: None,
                    };
                    let runs = if let Some(marked_range) = input.marked_range.as_ref() {
                        vec![
                            TextRun {
                                len: marked_range.start,
                                ..run.clone()
                            },
                            TextRun {
                                len: marked_range.end - marked_range.start,
                                underline: Some(UnderlineStyle {
                                    color: Some(run.color),
                                    thickness: px(1.0),
                                    wavy: false,
                                }),
                                ..run.clone()
                            },
                            TextRun {
                                len: text.len() - marked_range.end,
                                ..run.clone()
                            },
                        ]
                        .into_iter()
                        .filter(|run| run.len > 0)
                        .collect()
                    } else {
                        vec![run]
                    };

                    let font_size = style.font_size.to_pixels(window.rem_size());
                    window
                        .text_system()
                        .shape_line(text.clone().into(), font_size, &runs, None)
                })
                .collect::<Vec<_>>();

            let mut sum = 0;
            let mut lines_length = Vec::new();
            for line in lines.iter() {
                lines_length.push((line, sum));
                sum += line.text.len() + 1;
            }

            // Find the line that contains the cursor
            let cursor_pos = lines_length
                .iter()
                .enumerate()
                .skip_while(|(_, (line, sum))| *sum + line.text.len() < cursor)
                .map(|(i, (line, sum))| (i, line.x_for_index(cursor - sum)))
                .next();

            let (selection, cursor) = if selected_range.is_empty() {
                let (cursor_line, cursor_pos) = cursor_pos.unwrap_or((0, px(0.)));
                (
                    vec![],
                    Some(fill(
                        Bounds::new(
                            point(
                                bounds.left() + cursor_pos,
                                bounds.top() + cursor_line * line_height,
                            ),
                            size(px(1.), line_height),
                        ),
                        theme.foreground,
                    )),
                )
            } else {
                (
                    lines_length
                        .iter()
                        .enumerate()
                        .filter_map(|(cursor_line, (line, sum))| {
                            let last_character = *sum + line.text.len();
                            if selected_range.start > last_character || selected_range.end < *sum {
                                None
                            } else {
                                Some(fill(
                                    Bounds::from_corners(
                                        point(
                                            bounds.left()
                                                + line.x_for_index(
                                                    selected_range
                                                        .start
                                                        .checked_sub(*sum)
                                                        .unwrap_or_default(),
                                                ),
                                            bounds.top() + cursor_line * line_height,
                                        ),
                                        point(
                                            bounds.left()
                                                + line.x_for_index(
                                                    selected_range
                                                        .end
                                                        .checked_sub(*sum)
                                                        .unwrap_or(last_character),
                                                ),
                                            bounds.top() + (cursor_line + 1) * line_height,
                                        ),
                                    ),
                                    Rgba {
                                        a: 0.5,
                                        ..theme.button_background
                                    },
                                ))
                            }
                        })
                        .collect(),
                    None,
                )
            };

            PrepaintState {
                lines,
                cursor,
                selection,
            }
        })
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        for selection in prepaint.selection.drain(..) {
            window.paint_quad(selection)
        }

        for (i, line) in prepaint.lines.iter().enumerate() {
            line.paint(
                bounds.origin + point(px(0.), window.line_height() * i),
                window.line_height(),
                TextAlign::Left,
                None,
                window,
                cx,
            )
            .unwrap();
        }

        if focus_handle.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor);
        }

        self.input.update(cx, |input, _cx| {
            input.last_layout = Some(prepaint.lines.clone());
            input.last_bounds = Some(bounds);
        });
    }
}

fn split_string_lines(string: &str) -> impl Iterator<Item = &str> {
    string.split('\n').map(|line| {
        let Some(line) = line.strip_suffix('\r') else {
            return line;
        };
        line
    })
}
