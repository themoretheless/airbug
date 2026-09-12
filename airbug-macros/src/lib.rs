#![forbid(unsafe_code)]
//! Optional syntax that expands to ordinary Rust types and #[test] functions.
use proc_macro::TokenStream;
use proc_macro2::TokenStream as Tokens;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, spanned::Spanned};

fn runtime() -> syn::Result<Tokens> {
    match proc_macro_crate::crate_name("airbug") {
        Ok(proc_macro_crate::FoundCrate::Itself) => Ok(quote!(::airbug)),
        Ok(proc_macro_crate::FoundCrate::Name(name)) => {
            let name = format_ident!("{}", name);
            Ok(quote!(::#name))
        }
        Err(error) => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            error.to_string(),
        )),
    }
}
fn finish(result: syn::Result<Tokens>) -> TokenStream {
    result.unwrap_or_else(syn::Error::into_compile_error).into()
}
/// Generate structs and field builders. Field options: #[fixture(default)] or #[fixture(with = function)].
#[proc_macro_derive(Generate, attributes(fixture))]
pub fn generate(input: TokenStream) -> TokenStream {
    finish(derive_generate(parse_macro_input!(
        input as syn::DeriveInput
    )))
}
fn derive_generate(input: syn::DeriveInput) -> syn::Result<Tokens> {
    use syn::ext::IdentExt;
    let root = runtime()?;
    let name = &input.ident;
    let vis = &input.vis;
    if input.generics.lifetimes().next().is_some() {
        return Err(syn::Error::new(
            input.generics.span(),
            "Generate requires owned 'static structs; lifetime parameters are not supported",
        ));
    }
    for attr in &input.attrs {
        if attr.path().is_ident("fixture") {
            return Err(syn::Error::new_spanned(
                attr,
                "fixture options belong on fields",
            ));
        }
    }
    let syn::Data::Struct(data) = &input.data else {
        return Err(syn::Error::new(
            name.span(),
            "Generate supports structs; implement Generate explicitly for enums and unions",
        ));
    };
    let mut generics = input.generics.clone();
    let (_, original_types, _) = input.generics.split_for_impl();
    generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#name #original_types: 'static));
    let mut fields = Vec::new();
    let mut values = Vec::new();
    let mut stored = Vec::new();
    let mut setters = Vec::new();
    let mut defaults = Vec::new();
    let mut build_values = Vec::new();
    for (index, field) in data.fields.iter().enumerate() {
        let ty = &field.ty;
        let member = field.ident.as_ref().map(|n| quote!(#n)).unwrap_or_else(|| {
            let n = syn::Index::from(index);
            quote!(#n)
        });
        let slot = format_ident!("__airbug_value_{}", index);
        let setter = field
            .ident
            .as_ref()
            .map(|n| format_ident!("with_{}", n.unraw()))
            .unwrap_or_else(|| format_ident!("with_{}", index));
        let mut default = false;
        let mut factory: Option<syn::Path> = None;
        for attr in &field.attrs {
            if attr.path().is_ident("fixture") {
                attr.parse_nested_meta(|meta| {
                    if default || factory.is_some() {
                        return Err(meta.error("use exactly one fixture strategy per field"));
                    }
                    if meta.path.is_ident("default") {
                        default = true;
                        Ok(())
                    } else if meta.path.is_ident("with") {
                        factory = Some(meta.value()?.parse()?);
                        Ok(())
                    } else {
                        Err(meta.error("expected default or with = function"))
                    }
                })?;
            }
        }
        let value = if default {
            generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#ty: ::core::default::Default));
            quote!(<#ty as ::core::default::Default>::default())
        } else if let Some(factory) = factory {
            quote!(#factory(__airbug_ctx)?)
        } else {
            generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#ty: #root::Generate));
            quote!(__airbug_ctx.try_build::<#ty>()?)
        };
        fields.push(quote!(#member: #value));
        values.push(quote!(#slot));
        stored.push(quote!(#slot: ::core::option::Option<#ty>));
        defaults.push(quote!(#slot: ::core::option::Option::None));
        build_values.push(quote!(#member: match #slot { ::core::option::Option::Some(value) => value, ::core::option::Option::None => #value }));
        let field_vis = &field.vis;
        setters.push(quote!(#field_vis fn #setter(mut self, value: #ty) -> Self { self.#slot = ::core::option::Option::Some(value); self }));
    }
    let builder = format_ident!("{}FixtureBuilder", name.unraw());
    let mut builder_generics = generics.clone();
    builder_generics
        .params
        .insert(0, parse_quote!('__airbug_ctx));
    let (impl_g, types, where_g) = generics.split_for_impl();
    let (builder_impl, builder_types, builder_where) = builder_generics.split_for_impl();
    let args: Vec<_> = generics
        .params
        .iter()
        .map(|p| match p {
            syn::GenericParam::Type(p) => {
                let id = &p.ident;
                quote!(#id)
            }
            syn::GenericParam::Const(p) => {
                let id = &p.ident;
                quote!(#id)
            }
            syn::GenericParam::Lifetime(_) => unreachable!(),
        })
        .collect();
    Ok(quote! {
        impl #impl_g #root::Generate for #name #types #where_g {
            fn generate(__airbug_ctx: &mut #root::FixtureContext) -> ::core::result::Result<Self, #root::GenerationError> {
                ::core::result::Result::Ok(Self { #(#fields),* })
            }
        }
        #[must_use = "call build or try_build to generate the object"]
        #vis struct #builder #builder_generics #builder_where {
            __airbug_context: &'__airbug_ctx mut #root::FixtureContext,
            #(#stored),*
        }
        impl #impl_g #root::fixture::FixtureBuild for #name #types #where_g {
            type Builder<'__airbug_ctx> = #builder<'__airbug_ctx, #(#args),*>;
            fn fixture_builder(ctx: &mut #root::FixtureContext) -> Self::Builder<'_> {
                #builder { __airbug_context: ctx, #(#defaults),* }
            }
        }
        impl #builder_impl #builder #builder_types #builder_where {
            #(#setters)*
            #vis fn try_build(self) -> ::core::result::Result<#name #types, #root::GenerationError> {
                let Self { __airbug_context, #(#values),* } = self;
                __airbug_context.try_build_with(|__airbug_ctx| ::core::result::Result::Ok(#name { #(#build_values),* }))
            }
            #[track_caller]
            #vis fn build(self) -> #name #types {
                self.try_build().unwrap_or_else(|error| panic!("{error}"))
            }
        }
    })
}

/// Generate MockTrait with a method mock per trait method. Owned returns only.
#[proc_macro_attribute]
pub fn mock(attr: TokenStream, input: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return syn::Error::new(proc_macro2::Span::call_site(), "mock takes no arguments")
            .into_compile_error()
            .into();
    }
    finish(derive_mock(parse_macro_input!(input as syn::ItemTrait)))
}
fn derive_mock(input: syn::ItemTrait) -> syn::Result<Tokens> {
    use syn::visit::Visit;
    struct Unsupported(bool);
    impl<'ast> Visit<'ast> for Unsupported {
        fn visit_type_reference(&mut self, _: &'ast syn::TypeReference) {
            self.0 = true;
        }
        fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
            if node.path.segments.iter().any(|s| s.ident == "Self") {
                self.0 = true;
            }
            syn::visit::visit_type_path(self, node);
        }
        fn visit_type_impl_trait(&mut self, _: &'ast syn::TypeImplTrait) {
            self.0 = true;
        }
    }
    let root = runtime()?;
    if !input.generics.params.is_empty()
        || input.generics.where_clause.is_some()
        || input.unsafety.is_some()
    {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "mock supports safe traits without generic parameters or where clauses",
        ));
    }
    if !input.supertraits.iter().all(|b| matches!(b, syn::TypeParamBound::Trait(t) if t.path.is_ident("Send") || t.path.is_ident("Sync"))) {
        return Err(syn::Error::new_spanned(&input.supertraits, "only Send and Sync supertraits are supported"));
    }
    let name = &input.ident;
    let mock_name = format_ident!("Mock{}", name);
    let vis = &input.vis;
    let mut fields = Vec::new();
    let mut constructors = Vec::new();
    let mut methods = Vec::new();
    let mut checks = Vec::new();
    for item in &input.items {
        let syn::TraitItem::Fn(method) = item else {
            return Err(syn::Error::new_spanned(
                item,
                "mock supports methods only; associated types and constants require a manual adapter",
            ));
        };
        let sig = &method.sig;
        if !sig.generics.params.is_empty()
            || sig.generics.where_clause.is_some()
            || sig.unsafety.is_some()
            || sig.abi.is_some()
            || sig.variadic.is_some()
        {
            return Err(syn::Error::new_spanned(
                sig,
                "mock methods must be safe Rust methods without generics or variadics",
            ));
        }
        let Some(syn::FnArg::Receiver(receiver)) = sig.inputs.first() else {
            return Err(syn::Error::new_spanned(
                sig,
                "mock methods require &self or &mut self",
            ));
        };
        if receiver.reference.is_none() || receiver.colon_token.is_some() {
            return Err(syn::Error::new_spanned(
                receiver,
                "mock methods require &self or &mut self",
            ));
        }
        for attr in &method.attrs {
            if !attr.path().is_ident("doc") {
                return Err(syn::Error::new_spanned(
                    attr,
                    "method attributes other than doc require a manual mock adapter",
                ));
            }
        }
        let id = &sig.ident;
        let result: syn::Type = match &sig.output {
            syn::ReturnType::Default => parse_quote!(()),
            syn::ReturnType::Type(_, ty) => (**ty).clone(),
        };
        let mut unsupported = Unsupported(false);
        unsupported.visit_type(&result);
        if unsupported.0 {
            return Err(syn::Error::new_spanned(
                &sig.output,
                "mock requires owned returns without Self or impl Trait",
            ));
        }
        let mut arg_types = Vec::new();
        let mut arg_values = Vec::new();
        let mut impl_sig = sig.clone();
        for (index, arg) in impl_sig.inputs.iter_mut().skip(1).enumerate() {
            let syn::FnArg::Typed(arg) = arg else {
                unreachable!()
            };
            let local = format_ident!("__airbug_arg_{index}");
            *arg.pat = parse_quote!(#local);
            let (owned, value) = if let syn::Type::Reference(reference) = &*arg.ty {
                if reference.mutability.is_some() {
                    return Err(syn::Error::new_spanned(
                        &arg.ty,
                        "mutable argument references require a manual mock adapter",
                    ));
                }
                let ty = &reference.elem;
                let mut unsupported = Unsupported(false);
                unsupported.visit_type(ty);
                if unsupported.0 {
                    return Err(syn::Error::new_spanned(
                        ty,
                        "nested references and Self arguments require a manual mock adapter",
                    ));
                }
                (
                    quote!(<#ty as ::std::borrow::ToOwned>::Owned),
                    quote!(::std::borrow::ToOwned::to_owned(#local)),
                )
            } else {
                let ty = &arg.ty;
                let mut unsupported = Unsupported(false);
                unsupported.visit_type(ty);
                if unsupported.0 {
                    return Err(syn::Error::new_spanned(
                        ty,
                        "nested references and Self arguments require a manual mock adapter",
                    ));
                }
                (quote!(#ty), quote!(#local))
            };
            arg_types.push(owned);
            arg_values.push(value);
        }
        fields.push(quote!(pub #id: #root::Mock<(#(#arg_types,)*), #result>));
        constructors
            .push(quote!(#id: #root::Mock::new(concat!(stringify!(#name), "::", stringify!(#id)))));
        methods.push(quote!(#impl_sig { self.#id.call((#(#arg_values,)*)) }));
        checks.push(quote!(if let ::core::result::Result::Err(error) = self.#id.verify() { errors.push(error); }));
    }
    let cfg: Vec<_> = input
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("cfg") || a.path().is_ident("cfg_attr"))
        .collect();
    Ok(quote! {
        #input
        #(#cfg)*
        #[derive(Clone)]
        #vis struct #mock_name { #(#fields),* }
        #(#cfg)*
        impl ::core::default::Default for #mock_name {
            fn default() -> Self { Self { #(#constructors),* } }
        }
        #(#cfg)*
        impl #name for #mock_name { #(#methods)* }
        #(#cfg)*
        impl #root::mock::VerifyMocks for #mock_name {
            fn verify_mocks(&self) -> ::core::result::Result<(), #root::mock::VerificationErrors> {
                let mut errors = ::std::vec::Vec::new(); #(#checks)*
                if errors.is_empty() { ::core::result::Result::Ok(()) }
                else { ::core::result::Result::Err(#root::mock::VerificationErrors(errors)) }
            }
        }
    })
}

struct Case {
    name: syn::Ident,
    arguments: syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>,
}
impl syn::parse::Parse for Case {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let name = input.parse()?;
        let content;
        syn::parenthesized!(content in input);
        Ok(Self {
            name,
            arguments: content.parse_terminated(syn::Expr::parse, syn::Token![,])?,
        })
    }
}
/// Named parameterized cases, each emitted as an ordinary #[test].
#[proc_macro_attribute]
pub fn cases(attr: TokenStream, input: TokenStream) -> TokenStream {
    let cases = parse_macro_input!(attr with syn::punctuated::Punctuated::<Case, syn::Token![,]>::parse_terminated);
    finish(expand_cases(
        cases,
        parse_macro_input!(input as syn::ItemFn),
    ))
}
fn expand_cases(
    cases: syn::punctuated::Punctuated<Case, syn::Token![,]>,
    mut input: syn::ItemFn,
) -> syn::Result<Tokens> {
    if cases.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.sig,
            "cases requires at least one named case",
        ));
    }
    let sig = &input.sig;
    if sig.asyncness.is_some()
        || sig.unsafety.is_some()
        || !sig.generics.params.is_empty()
        || sig.generics.where_clause.is_some()
        || sig.abi.is_some()
        || sig.variadic.is_some()
    {
        return Err(syn::Error::new_spanned(
            sig,
            "cases requires a synchronous safe function without generics",
        ));
    }
    if sig
        .inputs
        .iter()
        .any(|a| matches!(a, syn::FnArg::Receiver(_)))
    {
        return Err(syn::Error::new_spanned(
            sig,
            "cases supports free functions only",
        ));
    }
    let name = &sig.ident;
    let output = &sig.output;
    let mut seen = std::collections::HashSet::new();
    let mut wrappers = Vec::new();
    for case in cases {
        if !seen.insert(case.name.to_string()) {
            return Err(syn::Error::new_spanned(case.name, "duplicate case name"));
        }
        if case.arguments.len() != sig.inputs.len() {
            return Err(syn::Error::new_spanned(
                case.name,
                "case argument count does not match function parameters",
            ));
        }
        let id = case.name;
        let args = case.arguments;
        let attrs: Vec<_> = input
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("ignore") || a.path().is_ident("should_panic"))
            .collect();
        wrappers.push(quote!(#(#attrs)* #[test] fn #id() #output { super::#name(#args) }));
    }
    for attr in &input.attrs {
        if !["doc", "cfg", "ignore", "should_panic", "allow"]
            .iter()
            .any(|id| attr.path().is_ident(id))
        {
            return Err(syn::Error::new_spanned(
                attr,
                "cases emits #[test] itself; supported attributes: doc, cfg, allow, ignore, should_panic",
            ));
        }
    }
    let cfg: Vec<_> = input
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("cfg"))
        .cloned()
        .collect();
    input
        .attrs
        .retain(|a| !a.path().is_ident("ignore") && !a.path().is_ident("should_panic"));
    Ok(quote!(#[cfg(test)] #input #(#cfg)* #[cfg(test)] mod #name { use super::*; #(#wrappers)* }))
}
