use super::*;

use ::core::ops::{Range, RangeFull};

/// "Postfix [`extend()`][`Extend::extend()`]"".
pub(crate)
trait PourIntoExt : Sized + IntoIterator {
    fn pour_into(
        self,
        collection: &mut impl Extend<Self::Item>,
    )
    {
        collection.extend(self)
    }
}
impl<I : IntoIterator> PourIntoExt for I {}

/// Newtype so as not to forget to pick one of [`SpanLocationExt::location()`],
/// [`SpanLocationExt::hygiene()`], [`SpanLocationExt::location_and_hygiene()`]
/// when extracting a [`.span()`][`Spanned::span()`].
#[derive(Clone, Copy)]
pub(crate)
struct ExplicitSpan {
    pub(crate) bare: Span,
}

impl ExplicitSpan {
    pub(crate)
    fn bare(self) -> Span {
        self.bare
    }

    #[allow(unused)]
    pub(crate)
    fn mixed_site() -> Self {
        Self { bare: Span::mixed_site() }
    }
}

pub(crate)
trait SpanLocationExt : Copy {
    /// Extract *only* the location information of a span, leaving out
    /// its semantical / resolution information (which shall thus default
    /// to `Span::mixed_site()`, _i.e._, proper `macro_rules!` hygiene).
    ///
    /// This should reduce the chance of improper hygiene usage when involving
    /// spans only for its diagnostical properties, as well as reduce the chance
    /// for style/opinionated lints to fire, provided they've been correctly implemented
    /// to guard against machine/macro-generated code.
    ///
    /// Alas, some lints are not that good, and on the contrary, can get worsened by this.
    ///
    /// Hence why `.location()` is not implicitly and automatically used in the following
    /// `…quote_spanned!` macros, even if ideally they would.
    ///
    /// It is up to the invocation sites to remember to use `.span().location()` when
    /// extracting spans as often as possible.
    fn location(self) -> ExplicitSpan;

    /// Unused, here just for consistency.
    #[allow(unused)]
    fn hygiene(self) -> ExplicitSpan;

    #[allow(unused)]
    fn location_and_hygiene(self) -> ExplicitSpan;
}

impl SpanLocationExt for Span {
    fn location(self) -> ExplicitSpan {
        ExplicitSpan { bare: Span::mixed_site().located_at(self) }
    }

    /// Unused, here just for consistency.
    #[allow(unused)]
    fn hygiene(self) -> ExplicitSpan {
        ExplicitSpan { bare: Span::mixed_site().resolved_at(self) }
    }

    fn location_and_hygiene(self) -> ExplicitSpan {
        ExplicitSpan { bare: self }
    }
}

pub(crate)
trait SpanRange<Disambiguator> {
    fn span_range(&self) -> Range<Span>;
}

impl SpanRange<()> for RangeFull {
    fn span_range(&self) -> Range<Span> {
        Span::mixed_site().span_range()
    }
}

impl SpanRange<()> for Span {
    fn span_range(&self) -> Range<Span> {
        let &span = self;
        span..span
    }
}

impl SpanRange<()> for Range<Span> {
    fn span_range(&self) -> Self {
        self.clone()
    }
}

impl<T : ?Sized + ToTokens> SpanRange<&dyn ToTokens> for T {
    fn span_range(&self) -> Range<Span> {
        let mut tts = self.to_token_stream().into_iter().map(|tt| tt.span());
        let first = tts.next().unwrap_or_else(Span::mixed_site);
        first..tts.last().unwrap_or(first)
    }
}

pub(crate)
fn compile_warning<S : ?Sized + SpanRange<impl Sized>>(
    spans: &S,
    message: &str,
) -> TokenStream2
{

    let Range { start, end } = spans.span_range();
    let ref message = ["\n\n", message].concat();
    let warning = Ident::new("custom_warning", start);
    quote_spanned!(end.location()=>
        #[allow(nonstandard_style, clippy::all)]
        const _: () = {
            #[allow(nonstandard_style)]
            struct implied_bounds_ {
                #[deprecated(note = #message)]
                #warning: ()
            }
            //                        start     end
            let _ = implied_bounds_ { #warning: () };
            //                        ^^^^^^^^^^^^
        };
    )
}

// -- Make `rust-analyzer` suggested parenthesized macro invocations. --
//    And also force the `ExplicitSpan` nudge.

/// ```rust ,ignore
/// quote_spanned!(..)
/// ```
#[allow(unused)]
macro_rules! quote_spanned {( $explicit_span:expr=> $($tt:tt)* ) => (
    ::quote::quote_spanned!(
        $crate::utils::ExplicitSpan::bare($explicit_span)=>
        $($tt)*
    )
)}
pub(crate) use quote_spanned;

/// ```rust ,ignore
/// quote!(..)
/// ```
#[allow(unused)]
macro_rules! quote {( $($tt:tt)* ) => (
    ::quote::quote_spanned!(Span::mixed_site()=> $($tt)* )
)}
pub(crate) use quote;

/// ```rust ,ignore
/// parse_quote_spanned!(..)
/// ```
#[allow(unused)]
macro_rules! parse_quote_spanned {( $explicit_span:expr=> $($tt:tt)* ) => (
    ::syn::parse_quote_spanned!(
        $crate::utils::ExplicitSpan::bare($explicit_span)=>
        $($tt)*
    )
)}
pub(crate) use parse_quote_spanned;

/// ```rust ,ignore
/// parse_quote!(..)
/// ```
#[allow(unused)]
macro_rules! parse_quote {( $($tt:tt)* ) => (
    ::syn::parse_quote_spanned!(Span::mixed_site()=> $($tt)* )
)}
pub(crate) use parse_quote;
