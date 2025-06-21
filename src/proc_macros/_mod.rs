//! Crate not intended for direct use.
//! Use https:://docs.rs/implied-bounds instead.
// Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template
#![allow(nonstandard_style, unused_imports, unused_braces)]

use ::core::{
    mem,
    ops::Not as _,
};
use ::proc_macro::{
    TokenStream,
};
use ::proc_macro2::{
    Span,
    TokenStream as TokenStream2,
    TokenTree as TT,
};
use ::quote::{
    format_ident,
    ToTokens,
};
use ::syn::{*,
    parse::{Parse, Parser, ParseStream},
    punctuated::Punctuated,
    Result, // Explicitly shadow it
    spanned::Spanned,
};

use self::{
    args::{
        Args,
        Crate,
    },
    utils::{
        compile_warning,
        quote, quote_spanned,
        parse_quote, parse_quote_spanned,
        PourIntoExt,
        SpanLocationExt,
    },
};

mod args;
mod utils;

///
#[proc_macro_attribute] pub
fn implied_bounds(
    args: TokenStream,
    input: TokenStream,
) -> TokenStream
{
    implied_bounds_impl(args.into(), input.into())
    //  .map(|ret| { println!("{}", ret); ret })
        .unwrap_or_else(|err| {
            let mut errors =
                err .into_iter()
                    .map(|err| Error::new(
                        err.span(),
                        format_args!("`#[::implied_bounds::implied_bounds]`: {}", err),
                    ))
            ;
            let mut err = errors.next().unwrap();
            errors.for_each(|cur| err.combine(cur));
            err.to_compile_error()
        })
        .into()
}

fn implied_bounds_impl(
    args: TokenStream2,
    input: TokenStream2,
) -> Result<TokenStream2>
{
    let mut args: Args = parse2(args)?;
    let mut trait_: ItemTrait = parse2(input)?;

    let _guard = Crate::init(args.krate.take());

    let mut debugged_predicates = vec![];

    extract_non_implied_predicates(&mut trait_, &args, &mut debugged_predicates)
        .into_iter()
        .map(transform_into_equivalent_implied_predicate)
        .map(WherePredicate::Type)
        // Let's prepend rather than append since it appears to improve the diagnostics w.r.t. our
        // duplicated predicates.
        .chain(mem::take(&mut trait_.generics.make_where_clause().predicates))
        .pour_into(&mut trait_.generics.make_where_clause().predicates);

    let mut ret = trait_.into_token_stream();
    debugged_predicates.into_iter().flatten().pour_into(&mut ret);

    Ok(ret)
}

/// Locate and extract the non-implied predicates present in this `trait` definition.
///
///   - Either the bounds on a generic parameter, _e.g._, `trait Foo<T : Clone> …`;
///   - or the `where` predicates which do not have `Self` as the LHS / "bounded type".
///
/// ---
///
/// What about `GAT` where clauses?
///
/// ```rust ,ignore
/// type GatA<const B: bool> where Self::GatA<true> : Bounds;
/// ```
///
/// Well, consider:
///
/// ```rust ,ignore
/// type GatB<'a> where Self : 'a;
/// type GatC<T> where T : Copy;
/// ```
///
/// The latter is impossible to express in an entailed manner, since the actual semantics are:
///
/// ```rust ,ignore
/// type GatB<'a where Self : 'a>;
/// type GatC<T : Copy>;
/// ```
///
/// Maybe there is a simple mechanical/algorithmic way for syntactic heuristics to distinguish
/// between the two. For now, this work is deemed not to be worth the effort; it shall be
/// up to the user to rewrite/move the `Self::GatA<true> : Bounds` predicate from GAT position
/// to "`trait` `where` clause" position.
///
/// ---
///
/// This extraction is `take()`-like, as in, it *strips* the trait of these, mutating it.
///
///   - (except when the predicates are repeatable, in which case a copy of the original predicates
///     are left in place, "untouched", for the sake of diagnostics).
///
/// It shall be the role of the caller of this function to transform the so extracted predicates
/// into their implied/entailed form, as "super traits" / `Self :`-bounding clauses involving
/// an interior assoc type bound (see `::implied_bounds::ImpliedPredicate`'s docs for more info).
fn extract_non_implied_predicates(
    trait_: &mut ItemTrait,
    args: &Args,
    debugged_predicates: &mut Vec<TokenStream2>,
) -> Vec<PredicateType>
{
    let mut ret = vec![];
    let mut found_clause = false;
    let debug_report_clause: &mut dyn FnMut(&dyn ToTokens) = if args.debug.is_some() {
        &mut |tts| {
            found_clause = true;
            debugged_predicates.push(
                compile_warning(tts, "[debug] this predicate is not implied, adjusting it…")
            );
        }
    } else {
        &mut |_| {
            found_clause = true;
        }
    };
    trait_.generics.params.iter_mut().filter_map(|param_intro| {
        let GenericParam::Type(param_intro) = param_intro else { return None };
        let bounds = mem::take(&mut param_intro.bounds);
        if bounds.is_empty() {
            return None;
        }
        // Non-implied bounds.

        debug_report_clause(&bounds);
        if may_be_higher_ranked(&bounds).not() {
            // a non-higher-ranked clause shall not involve a higher-ranked assoc type;
            // which allows duplicating it.
            // We thus try to do that duplication unless potentially non-applicable,
            // so as to improve the diagnostics:
            // > `X` is not `Send`
            // rather than:
            // > `<Self as …ImpliedPredicate<X>>::Impls` is not `Send`
            param_intro.bounds.clone_from(&bounds);
        }
        Some(PredicateType {
            lifetimes: None,
            bounded_ty: {
                let T @ _ = &param_intro.ident;
                parse_quote!( #T )
            },
            colon_token: param_intro.colon_token?,
            bounds,
        })
    }).pour_into(&mut ret);
    if let Some(mut where_clause) = trait_.generics.where_clause.take() {
        let mut retained_predicates = Vec::with_capacity(where_clause.predicates.len());
        where_clause.predicates.into_iter().filter_map(|predicate| {
            match predicate {
                // Handle `BoundedType : …` predicates…
                | WherePredicate::Type(predicate)
                if  predicate.bounds.is_empty().not()
                    // …so long as the `BoundedType` not be `Self` (since that is
                    // a special synonym for a super-trait, rather than a mere clause).
                    &&  matches!(
                            &predicate.bounded_ty,
                            // Note: this mistakenly misses `(Self)`, but there is only
                            // so much we can do syntactically (e.g., quid of `m!(Self)`),
                            // and it can sometimes be a handy opt-out of this branch
                            // for those wanting to experiment with the difference between
                            // a "mere (entailed) clause" and a super-trait (e.g. how it
                            // affects `dyn`-ability)
                            Type::Path(TypePath {
                                qself: None,
                                path: Self_,
                            })
                            if Self_.is_ident("Self")
                        )
                        .not()
                => {
                    // Non-implied predicate.
                    debug_report_clause(&predicate);

                    if (
                        predicate.lifetimes.as_ref().is_some_and(|it| it.lifetimes.is_empty().not())
                        ||
                        may_be_higher_ranked(&predicate.bounds)
                    ).not()
                    {
                        // See previous `may_be_higher_ranked()` usage above.
                        retained_predicates.push(WherePredicate::Type(predicate.clone()));
                    }

                    Some(predicate)
                },
                | _ => {
                    retained_predicates.push(predicate);
                    None
                },
            }
        }).pour_into(&mut ret);
        where_clause.predicates = retained_predicates.into_iter().collect();
        trait_.generics.where_clause = Some(where_clause);
    }

    if args.allow_none.is_none() && found_clause.not() {
        debugged_predicates.push(compile_warning(
            &..,
            "No non-implied clauses found for this trait, you may skip using this macro altogether.\
            \n\n\
            To silence this warning, use `#[…implied_bounds(allow_none, …)]`.",
        ));
    }

    ret
}

/// Transform `#bounded_ty : #bounds` into:
///
/// ```rust ,ignore
/// Self : ImpliedPredicate<#bounded_ty, Impls : #bounds>
/// ```
///
/// (modulo robust pathing, and `for<>` quantification).
fn transform_into_equivalent_implied_predicate(
    mut predicate: PredicateType
) -> PredicateType
{
    // Span red tape.
    let opening_span = predicate.span().location();
    let closing_span =
        predicate
            .bounds
            .pairs()
            .last()
            .and_then(|pair| pair.to_token_stream().into_iter().last())
            .unwrap()
            .span()
            .location()
    ;
    let closing_span_angle_bracket = quote_spanned!(closing_span=>
        >
    );

    let krate = Crate::get().unwrap_or_else(|| quote_spanned!(opening_span=>
        ::implied_bounds
    ));

    // Replace the original LHS / `bounded_ty` of the predicate with `Self` so that
    // this part be entailed.
    let bounded_ty = mem::replace(
        &mut predicate.bounded_ty,
        parse_quote_spanned!(opening_span=> Self ),
    );
    let bounds = predicate.bounds;
    // Use the `ImpliedPredicate` trick to now express, *in an entailed manner*, that
    // `#bounded_ty : #bounds`.
    predicate.bounds = parse_quote_spanned!(opening_span=>
        #krate::ImpliedPredicate<
            #bounded_ty,
            Impls : #bounds // ,
        #closing_span_angle_bracket // >
    );

    predicate
}

fn may_be_higher_ranked(
    bounds: &Punctuated<TypeParamBound, Token![+]>,
) -> bool
{
    bounds.iter().any(|bound| {
        let TypeParamBound::Trait(bound) = bound else { return false };
        // Do we have `: for<'…> …`?
        bound.lifetimes.as_ref().is_some_and(|it| it.lifetimes.is_empty().not())
        ||
        // or `: Fn…(…)` (the latter is very coarse, not all `Fn…` trait clauses
        // are higher-ranked, but since the "bound repetition" is merely there to improve
        // diagnostics, I don't think it warrants the effort of trying to fully visit
        // an `Fn` clause in order to determine whether its signature is actually higher-ranked).
        matches!(
            bound.path.segments.last().unwrap().arguments,
            PathArguments::Parenthesized { .. },
        )
    })
}
