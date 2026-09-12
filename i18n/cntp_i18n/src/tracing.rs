macro_rules! define_tracing_i18n_macros {
    (
        $dollar:tt;
        $(
            $tr_macro:ident,
            $trn_macro:ident,
            $tracing_macro:path;
        )*
    ) => {
        $(
            #[macro_export]
            macro_rules! $tr_macro {
                ($dollar($dollar body:tt)*) => {{
                    let resolved = $crate::tr!($dollar($dollar body)*);
                    $tracing_macro!("{}", resolved)
                }};
            }

            #[macro_export]
            macro_rules! $trn_macro {
                ($dollar($dollar body:tt)*) => {{
                    let resolved = $crate::trn!($dollar($dollar body)*);
                    $tracing_macro!("{}", resolved)
                }};
            }

            pub use $tr_macro;
            pub use $trn_macro;
        )*
    };
}

define_tracing_i18n_macros! {
    $;
    tr_trace, trn_trace, tracing::trace;
    tr_debug, trn_debug, tracing::debug;
    tr_info,  trn_info,  tracing::info;
    tr_warn,  trn_warn,  tracing::warn;
    tr_error, trn_error, tracing::error;
}
