use super::*;

use ::core::cell::RefCell;

mod kw {
    ::syn::custom_keyword!(allow_none);
    ::syn::custom_keyword!(debug);
}

#[derive(Default)]
pub(crate)
struct Args {
    pub(crate)
    debug: Option<kw::debug>,

    pub(crate)
    allow_none: Option<kw::allow_none>,

    pub(crate)
    krate: Option<Path>,
}

const USAGE: &str = r#"Usage:

#[implied_bounds(
    // [Optional] Whether to disable the warning about lack of non-implied clauses.
    allow_none,

    // [Optional] Highlight every non-implied clause (via deprecation warnings).
    debug,

    // [Optional] Override `::implied_bounds::…` paths in the expansion with `$(::)? some::path::…`.
    //            Useful when `macro_rules!` or middle-libs are involved, and the `::implied_bounds`
    //            path is no longer (directly, and syntactically) reachable.
    crate = $(::)? some::path,
)]
"#;

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> Result<Args> {
        || -> Result<_> {
            let mut ret = Args::default();
            let lookahead = input.lookahead1();
            while input.is_empty().not() {
                match () {
                    | _case if lookahead.peek(kw::debug) => {
                        if ret.debug.is_some() {
                            return Err(input.error("duplicate arg"));
                        }
                        ret.debug = Some(input.parse().unwrap());
                    },
                    | _case if lookahead.peek(kw::allow_none) => {
                        if ret.allow_none.is_some() {
                            return Err(input.error("duplicate arg"));
                        }
                        ret.allow_none = Some(input.parse().unwrap());
                    },
                    | _case if lookahead.peek(Token![crate]) => {
                        if ret.krate.is_some() {
                            return Err(input.error("duplicate arg"));
                        }
                        let _: Token![crate] = input.parse().unwrap();
                        let _: Token![=] = input.parse()?;
                        ret.krate = Some(Path::parse_mod_style(input)?);
                    },
                    | _default => return Err(lookahead.error()),
                }
                let _: Option<Token![,]> = input.parse()?;
            }
            Ok(ret)
        }().map_err(|mut err| {
            let usage = Error::new(Span::mixed_site(), USAGE);
            err.combine(usage);
            err
        })
    }
}

pub(crate)
struct Crate;

impl Crate {
    #[must_use = "this is a scope-defining guard which clears the thread local on drop, bind it."]
    pub(crate)
    fn init(
        path: Option<Path>,
    ) -> impl Sized {
        let path = path?;

        CRATE.with_borrow_mut(|krate| {
            *krate = Some(path);
        });

        struct Guard {}

        impl Drop for Guard {
            fn drop(&mut self) {
                CRATE.with_borrow_mut(|krate| {
                    *krate = None;
                })
            }
        }

        Some(Guard {})
    }

    pub(crate)
    fn get() -> Option<TokenStream2> {
        CRATE.with_borrow(|c| c.as_ref().map(ToTokens::to_token_stream))
    }
}

thread_local! {
    static CRATE: RefCell<Option<Path>> = const { RefCell::new(None) };
}
