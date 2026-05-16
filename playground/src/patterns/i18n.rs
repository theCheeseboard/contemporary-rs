use chrono::{DateTime, Local};
use cntp_i18n::{LayoutDirection, ListFunction, Locale, LocaleFormattable, tr, trf, I18N_MANAGER};
use contemporary::components::constrainer::constrainer;
use contemporary::components::grandstand::grandstand;
use contemporary::components::layer::layer;
use contemporary::components::subtitle::subtitle;
use contemporary::components::text_field::TextField;
use contemporary::styling::theme::ThemeStorage;
use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, px,
};

pub struct I18n {
    i18n_language: Entity<TextField>,
    quote_strings_text_field: Entity<TextField>,
}

impl I18n {
    pub fn new(cx: &mut App) -> Entity<Self> {
        cx.new(|cx| I18n {
            i18n_language: cx.new(|cx| {
                let mut text_field = TextField::new("i18n-preferred-language", cx);
                text_field.set_text(I18N_MANAGER.locale().messages.first().unwrap().as_str());
                text_field.set_placeholder(
                    tr!("I18N_LANGUAGE_CODE", "Enter a language code")
                        .to_string()
                        .as_str(),
                );
                text_field
            }),
            quote_strings_text_field: cx.new(|cx| {
                let mut text_field = TextField::new("quote-strings-text", cx);
                text_field.set_placeholder(
                    tr!("QUOTE_STRINGS_PLACEHOLDER", "What's your favourite song?")
                        .to_string()
                        .as_str(),
                );
                text_field
            }),
        })
    }
}

impl Render for I18n {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let requested_locale = self.i18n_language.read(cx).text();
        let quote_strings_text = self.quote_strings_text_field.read(cx).text();

        let locale = Locale::new_from_locale_identifier(requested_locale);

        let local_time: DateTime<Local> = Local::now();
        let layout_direction = locale.layout_direction();

        let list_items = vec![
            tr!("LIST_EXAMPLE_APPLE", "Apple", #description = "A red fruit").to_string(),
            tr!("LIST_EXAMPLE_BANANA", "Banana").to_string(),
            tr!("LIST_EXAMPLE_CHERRY", "Cherry").to_string(),
            tr!("LIST_EXAMPLE_ORANGE", "Orange", #description = "A citrus fruit").to_string(),
        ];
        let two_list_items = vec![
            tr!("LIST_EXAMPLE_APPLE").to_string(),
            tr!("LIST_EXAMPLE_BANANA").to_string(),
        ];
        let standard_list = locale.build_list(&list_items);
        let or_list = locale
            .build_list(&list_items)
            .with_list_function(ListFunction::Or);
        let unit_list = locale
            .build_list(&list_items)
            .with_list_function(ListFunction::Unit);
        let standard_two_list = locale.build_list(&two_list_items);

        div()
            .bg(theme.background)
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .child(
                grandstand("i18n-grandstand")
                    .text(tr!("I18N_TITLE", "Locales"))
                    .pt(px(36.)),
            )
            .child(
                constrainer("i18n")
                    .flex()
                    .flex_col()
                    .w_full()
                    .p(px(8.))
                    .gap(px(8.))
                    .child(
                        layer()
                            .flex()
                            .flex_col()
                            .p(px(8.))
                            .w_full()
                            .child(subtitle(tr!("I18N_SETUP", "Locales")))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(8.))
                                    .child(tr!(
                                        "I18N_SETUP_DESCRIPTION",
                                        "Enter a locale code to see information about that locale."
                                    ))
                                    .child(self.i18n_language.clone())
                                    .child(tr!(
                                        "I18N_SELECTED_LANGUAGE",
                                        "Language: {{language}}",
                                        language = locale.human_readable_locale_name()
                                    ))
                                    .child(tr!(
                                        "I18N_LAYOUT_DIRECTION",
                                        "Layout Direction: {{layout_direction}}",
                                        layout_direction = match layout_direction {
                                            LayoutDirection::LeftToRight => tr!("LAYOUT_DIRECTION_LTR", "Left-to-Right"),
                                            LayoutDirection::RightToLeft => tr!("LAYOUT_DIRECTION_RTL", "Right-to-Left")
                                        })),
                            ),
                    )
                    .child(
                        layer()
                            .flex()
                            .flex_col()
                            .p(px(8.))
                            .w_full()
                            .child(subtitle(tr!("I18N_QUOTE_STRINGS", "Quote Strings")))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(8.))
                                    .child(tr!(
                                        "I18N_QUOTE_STRING_DESCRIPTION",
                                        "Type some text below to see it quoted."
                                    ))
                                    .child(self.quote_strings_text_field.clone())
                                    .child(tr!(
                                        "I18N_QUOTE_STRING_QUOTED",
                                        "Your favourite song is {{favourite_song}}",
                                        favourite_song:quote=quote_strings_text,
                                        #locale=&locale
                                    ))
                                    .child(tr!(
                                        "I18N_QUOTE_STRING_QUOTED",
                                        favourite_song:quote("alt")=quote_strings_text,
                                        #locale=&locale
                                    ))
                            ),
                    )
                    .child(
                        layer()
                            .flex()
                            .flex_col()
                            .p(px(8.))
                            .w_full()
                            .child(subtitle(tr!("I18N_NUMERIC_FORMATS", "Numeric Formats")))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(8.))
                                    .child(tr!(
                                        "I18N_NUMERIC_FORMAT_DESCRIPTION",
                                        "Numbers are formatted in the following manner in this locale:"
                                    ))
                                    .child(tr!(
                                        "I18N_NUMERIC_FORMAT_PI",
                                        "Pi: {{pi}}",
                                        pi =
                                            std::f64::consts::PI.to_locale_string(&locale)
                                    ))
                                    .child(tr!(
                                        "I18N_NUMERIC_FORMAT_NEGATIVE_E",
                                        "-e: {{e_neg}}",
                                        e_neg =
                                            (-std::f64::consts::E).to_locale_string(&locale)
                                    ))
                                    .child(tr!(
                                        "I18N_NUMERIC_FORMAT_SPEED_OF_LIGHT",
                                        "The speed of light: {{speed_of_light}} m/s",
                                        speed_of_light =
                                            299_792_458.to_locale_string(&locale)
                                    ))
                            ),
                    )
                    .child(
                        layer()
                            .flex()
                            .flex_col()
                            .p(px(8.))
                            .w_full()
                            .child(subtitle(tr!("I18N_DATE_TIME", "Date & Time")))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(8.))
                                    .child(tr!(
                                        "I18N_DATE_TIME_DESCRIPTION",
                                        "Dates are formatted in the following manner in this locale:"
                                    ))
                                    .child(tr!(
                                        "I18N_DATE_TIME_BASIC_MEDIUM",
                                        "YMD (medium): {{time}}",
                                        time:date=local_time,
                                        #locale=&locale
                                    ))
                                    .child(tr!(
                                        "I18N_DATE_TIME_BASIC_LONG",
                                        "YMD (short): {{time}}",
                                        time:date("YMD", length="short")=local_time,
                                        #locale=&locale
                                    ))
                                    .child(tr!(
                                        "I18N_DATE_TIME_ADV_SHORT",
                                        "Custom (short): {{time}}",
                                        time:date(date="E", length="short", time="minute")=local_time,
                                        #locale=&locale
                                    ))
                                    .child(tr!(
                                        "I18N_DATE_TIME_ADV_LONG",
                                        "Custom (long): {{time}}",
                                        time:date(date="MD", length="long", time="microsecond")=local_time,
                                        #locale=&locale
                                    ))
                                    .child(trf!(date(date="MD", length="long", time="microsecond"), quote, local_time))
                            ),
                    )
                    .child(
                        layer()
                            .flex()
                            .flex_col()
                            .p(px(8.))
                            .w_full()
                            .child(subtitle(tr!("I18N_LISTS", "Lists")))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(8.))
                                    .child(tr!(
                                        "I18N_LISTS_DESCRIPTION",
                                        "Lists are compiled in the following manner in this locale:"
                                    ))
                                    .child(tr!(
                                        "I18N_LISTS_STANDARD",
                                        "Standard: {{list}}",
                                        list = standard_list
                                    ))
                                    .child(tr!(
                                        "I18N_LISTS_OR",
                                        "Or: {{list}}",
                                        list = or_list
                                    ))
                                    .child(tr!(
                                        "I18N_LISTS_UNIT",
                                        "Unit: {{list}}",
                                        list = unit_list
                                    ))
                                    .child(tr!(
                                        "I18N_LISTS_STANDARD_2",
                                        "Standard with two items: {{list}}",
                                        list = standard_two_list
                                    ))
                            ),
                    ),
            )
    }
}
