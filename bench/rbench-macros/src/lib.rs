use proc_macro::TokenStream;
use quote::{format_ident, quote};
/// Register a safe synchronous function with no arguments through the Suite builder.
/// Enable `rbench`'s `macros` feature and call generated `register_NAME(&mut suite)`.
#[proc_macro_attribute]
pub fn bench(args: TokenStream, item: TokenStream) -> TokenStream {
    let f = syn::parse_macro_input!(item as syn::ItemFn);
    if !args.is_empty()
        || !f.sig.inputs.is_empty()
        || !f.sig.generics.params.is_empty()
        || f.sig.asyncness.is_some()
        || matches!(f.sig.safety, syn::Safety::Unsafe(_))
    {
        return syn::Error::new_spanned(&f.sig,"bench requires a safe synchronous nongeneric function with no arguments; use the builder for parameterized or async workloads").to_compile_error().into();
    }
    let name = &f.sig.ident;
    let register = format_ident!("register_{}", name);
    let visibility = &f.vis;
    quote!{#f #visibility fn #register(suite:&mut ::rbench::Suite<'_>){suite.bench(stringify!(#name),#name);}}.into()
}
