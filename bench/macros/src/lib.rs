use proc_macro::TokenStream;
use quote::{format_ident, quote};
/// Register a safe synchronous function with no arguments through the Suite builder.
/// Enable `airbug-bench`'s `macros` feature and call generated `register_NAME(&mut suite)`.
#[proc_macro_attribute]
pub fn bench(args: TokenStream, item: TokenStream) -> TokenStream {
    let f = syn::parse_macro_input!(item as syn::ItemFn);
    if !args.is_empty()
        || !f.sig.inputs.is_empty()
        || !f.sig.generics.params.is_empty()
        || f.sig.asyncness.is_some()
        || f.sig.unsafety.is_some()
    {
        return syn::Error::new_spanned(&f.sig,"bench requires a safe synchronous nongeneric function with no arguments; use the builder for parameterized or async workloads").to_compile_error().into();
    }
    let name = &f.sig.ident;
    let register = format_ident!("register_{}", name);
    let visibility = &f.vis;
    quote!{#f #visibility fn #register(suite:&mut ::airbug_bench::Suite<'_>){suite.bench(stringify!(#name),#name);}}.into()
}

/// Generate main and register the inline module's directly contained #[bench] functions.
#[proc_macro_attribute]
pub fn suite(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut module = syn::parse_macro_input!(item as syn::ItemMod);
    if !args.is_empty() {
        return syn::Error::new_spanned(&module.ident, "suite takes no arguments")
            .to_compile_error()
            .into();
    }
    let name = module.ident.clone();
    let Some((_, items)) = &mut module.content else {
        return syn::Error::new_spanned(&module, "suite requires an inline module")
            .to_compile_error()
            .into();
    };
    let mut registrations = Vec::new();
    for item in items.iter_mut() {
        let syn::Item::Fn(function) = item else {
            continue;
        };
        let Some(index) = function.attrs.iter().position(|attr| {
            let path = attr.path();
            path.is_ident("bench")
                || path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .eq(["airbug_bench", "bench"].into_iter().map(str::to_owned))
        }) else {
            continue;
        };
        let marker = function.attrs.remove(index);
        if !matches!(marker.meta, syn::Meta::Path(_))
            || !function.sig.inputs.is_empty()
            || !function.sig.generics.params.is_empty()
            || function.sig.asyncness.is_some()
            || function.sig.unsafety.is_some()
        {
            return syn::Error::new_spanned(
                &function.sig,
                "bench requires a safe synchronous nongeneric function with no arguments",
            )
            .to_compile_error()
            .into();
        }
        let ident = &function.sig.ident;
        let cfg = function
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("cfg"));
        registrations.push(quote! {
            #(#cfg)*
            { suite.bench(stringify!(#ident), #ident); }
        });
    }
    items.push(syn::parse_quote! {
        pub(super) fn __airbug_run_suite() -> ::airbug_bench::Result<()> {
            let mut suite = ::airbug_bench::Suite::new(stringify!(#name));
            #(#registrations)*
            suite.main()
        }
    });
    quote! {
        #module
        fn main() -> ::airbug_bench::Result<()> { #name::__airbug_run_suite() }
    }
    .into()
}
