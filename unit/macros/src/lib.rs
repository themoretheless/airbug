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
    use std::collections::{HashMap, HashSet};
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

    fn contains_ident(ty: &syn::Type, target: &syn::Ident) -> bool {
        struct Finder<'a>(&'a syn::Ident, bool);
        impl<'ast> Visit<'ast> for Finder<'_> {
            fn visit_ident(&mut self, node: &'ast syn::Ident) {
                if node == self.0 {
                    self.1 = true;
                }
            }
        }
        let mut finder = Finder(target, false);
        finder.visit_type(ty);
        finder.1
    }

    fn type_has_static_bound(
        bounds: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::Token![+]>,
    ) -> bool {
        bounds
            .iter()
            .any(|b| matches!(b, syn::TypeParamBound::Lifetime(lt) if lt.ident == "static"))
    }

    fn where_has_static_bound(where_clause: Option<&syn::WhereClause>, ident: &syn::Ident) -> bool {
        let Some(wc) = where_clause else {
            return false;
        };
        wc.predicates.iter().any(|pred| {
            let syn::WherePredicate::Type(tp) = pred else {
                return false;
            };
            let is_target = match &tp.bounded_ty {
                syn::Type::Path(path) => path.qself.is_none() && path.path.is_ident(ident),
                _ => false,
            };
            is_target && type_has_static_bound(&tp.bounds)
        })
    }

    fn rewrite_assoc_in_path_args(
        args: &mut syn::PathArguments,
        assoc_map: &HashMap<syn::Ident, syn::Ident>,
    ) {
        match args {
            syn::PathArguments::None => {}
            syn::PathArguments::AngleBracketed(ab) => {
                for arg in &mut ab.args {
                    match arg {
                        syn::GenericArgument::Type(t) => rewrite_assoc_in_type(t, assoc_map),
                        syn::GenericArgument::AssocType(at) => {
                            rewrite_assoc_in_type(&mut at.ty, assoc_map);
                        }
                        syn::GenericArgument::Constraint(c) => {
                            for bound in &mut c.bounds {
                                rewrite_assoc_in_bound(bound, assoc_map);
                            }
                        }
                        _ => {}
                    }
                }
            }
            syn::PathArguments::Parenthesized(par) => {
                for input in &mut par.inputs {
                    rewrite_assoc_in_type(input, assoc_map);
                }
                if let syn::ReturnType::Type(_, ret_ty) = &mut par.output {
                    rewrite_assoc_in_type(ret_ty, assoc_map);
                }
            }
        }
    }

    fn rewrite_assoc_in_bound(
        bound: &mut syn::TypeParamBound,
        assoc_map: &HashMap<syn::Ident, syn::Ident>,
    ) {
        if let syn::TypeParamBound::Trait(tr) = bound {
            for seg in &mut tr.path.segments {
                rewrite_assoc_in_path_args(&mut seg.arguments, assoc_map);
            }
        }
    }

    fn rewrite_assoc_in_type(ty: &mut syn::Type, assoc_map: &HashMap<syn::Ident, syn::Ident>) {
        match ty {
            syn::Type::Path(tp) => {
                if tp.qself.is_none()
                    && tp.path.segments.len() == 2
                    && tp.path.segments[0].ident == "Self"
                    && matches!(tp.path.segments[1].arguments, syn::PathArguments::None)
                    && let Some(mapped) = assoc_map.get(&tp.path.segments[1].ident)
                {
                    *ty = parse_quote!(#mapped);
                    return;
                }
                if let Some(qself) = &mut tp.qself {
                    let is_self_qself = matches!(
                        qself.ty.as_ref(),
                        syn::Type::Path(qp) if qp.qself.is_none() && qp.path.is_ident("Self")
                    );
                    if is_self_qself
                        && tp.path.segments.len() >= 2
                        && let Some(last) = tp.path.segments.last()
                        && matches!(last.arguments, syn::PathArguments::None)
                        && let Some(mapped) = assoc_map.get(&last.ident)
                    {
                        *ty = parse_quote!(#mapped);
                        return;
                    }
                    rewrite_assoc_in_type(&mut qself.ty, assoc_map);
                }
                for seg in &mut tp.path.segments {
                    rewrite_assoc_in_path_args(&mut seg.arguments, assoc_map);
                }
            }
            syn::Type::Reference(r) => rewrite_assoc_in_type(&mut r.elem, assoc_map),
            syn::Type::Ptr(p) => rewrite_assoc_in_type(&mut p.elem, assoc_map),
            syn::Type::Slice(s) => rewrite_assoc_in_type(&mut s.elem, assoc_map),
            syn::Type::Array(a) => rewrite_assoc_in_type(&mut a.elem, assoc_map),
            syn::Type::Tuple(t) => {
                for elem in &mut t.elems {
                    rewrite_assoc_in_type(elem, assoc_map);
                }
            }
            syn::Type::Paren(p) => rewrite_assoc_in_type(&mut p.elem, assoc_map),
            syn::Type::Group(g) => rewrite_assoc_in_type(&mut g.elem, assoc_map),
            syn::Type::BareFn(f) => {
                for input in &mut f.inputs {
                    rewrite_assoc_in_type(&mut input.ty, assoc_map);
                }
                if let syn::ReturnType::Type(_, ret) = &mut f.output {
                    rewrite_assoc_in_type(ret, assoc_map);
                }
            }
            syn::Type::TraitObject(obj) => {
                for bound in &mut obj.bounds {
                    rewrite_assoc_in_bound(bound, assoc_map);
                }
            }
            syn::Type::ImplTrait(it) => {
                for bound in &mut it.bounds {
                    rewrite_assoc_in_bound(bound, assoc_map);
                }
            }
            _ => {}
        }
    }

    fn rewrite_assoc_in_where(
        wc: &mut syn::WhereClause,
        assoc_map: &HashMap<syn::Ident, syn::Ident>,
    ) {
        for pred in &mut wc.predicates {
            if let syn::WherePredicate::Type(tp) = pred {
                rewrite_assoc_in_type(&mut tp.bounded_ty, assoc_map);
                for bound in &mut tp.bounds {
                    rewrite_assoc_in_bound(bound, assoc_map);
                }
            }
        }
    }

    let root = runtime()?;
    if input.unsafety.is_some() {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "mock supports safe traits only",
        ));
    }
    if let Some(lt) = input
        .generics
        .params
        .iter()
        .find(|p| matches!(p, syn::GenericParam::Lifetime(_)))
    {
        return Err(syn::Error::new_spanned(
            lt,
            "lifetime parameters on mocked traits require a manual mock adapter",
        ));
    }
    if !input.supertraits.iter().all(|b| {
        matches!(
            b,
            syn::TypeParamBound::Trait(t) if t.path.is_ident("Send") || t.path.is_ident("Sync")
        ) || matches!(b, syn::TypeParamBound::Lifetime(lt) if lt.ident == "static")
    }) {
        return Err(syn::Error::new_spanned(
            &input.supertraits,
            "only Send, Sync, and 'static supertraits are supported",
        ));
    }

    // Collect existing generic parameter names to avoid collisions when lowering associated types.
    let mut existing_idents: HashSet<String> = input
        .generics
        .params
        .iter()
        .filter_map(|p| match p {
            syn::GenericParam::Type(t) => Some(t.ident.to_string()),
            syn::GenericParam::Const(c) => Some(c.ident.to_string()),
            syn::GenericParam::Lifetime(_) => None,
        })
        .collect();

    struct AssocItemInfo {
        assoc_ident: syn::Ident,
        param_ident: syn::Ident,
        bounds: syn::punctuated::Punctuated<syn::TypeParamBound, syn::Token![+]>,
        default: Option<syn::Type>,
    }
    let mut assoc_items = Vec::new();
    let mut assoc_map: HashMap<syn::Ident, syn::Ident> = HashMap::new();

    for item in &input.items {
        match item {
            syn::TraitItem::Type(ty_item) => {
                if !ty_item.generics.params.is_empty() || ty_item.generics.where_clause.is_some() {
                    return Err(syn::Error::new_spanned(
                        ty_item,
                        "generic associated types (GATs) require a manual mock adapter",
                    ));
                }
                let param_ident = if existing_idents.insert(ty_item.ident.to_string()) {
                    ty_item.ident.clone()
                } else {
                    let candidate = format_ident!("__AirbugAssoc_{}", ty_item.ident);
                    existing_idents.insert(candidate.to_string());
                    candidate
                };
                assoc_map.insert(ty_item.ident.clone(), param_ident.clone());
                assoc_items.push(AssocItemInfo {
                    assoc_ident: ty_item.ident.clone(),
                    param_ident,
                    bounds: ty_item.bounds.clone(),
                    default: ty_item.default.as_ref().map(|(_, ty)| ty.clone()),
                });
            }
            syn::TraitItem::Const(c) => {
                return Err(syn::Error::new_spanned(
                    c,
                    "associated constants require a manual mock adapter (see airbug::mock::borrowed)",
                ));
            }
            syn::TraitItem::Fn(_) => {}
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "mock supports methods and associated types only; other trait items require a manual mock adapter",
                ));
            }
        }
    }

    let name = &input.ident;
    let mock_name = format_ident!("Mock{}", name);
    let vis = &input.vis;

    let mut mock_struct_generics = input.generics.clone();
    for info in &assoc_items {
        let param_ident = &info.param_ident;
        let bounds = &info.bounds;
        let default = &info.default;
        let tp: syn::TypeParam = if bounds.is_empty() {
            if let Some(def) = default {
                parse_quote!(#param_ident = #def)
            } else {
                parse_quote!(#param_ident)
            }
        } else if let Some(def) = default {
            parse_quote!(#param_ident: #bounds = #def)
        } else {
            parse_quote!(#param_ident: #bounds)
        };
        mock_struct_generics
            .params
            .push(syn::GenericParam::Type(tp));
    }
    // Strip non-trailing defaults on the struct declaration if any trailing type param lacks a default.
    let mut seen_non_default_or_const = false;
    for param in mock_struct_generics.params.iter_mut().rev() {
        match param {
            syn::GenericParam::Type(tp) => {
                if seen_non_default_or_const {
                    tp.default = None;
                    tp.eq_token = None;
                } else if tp.default.is_none() {
                    seen_non_default_or_const = true;
                }
            }
            syn::GenericParam::Const(cp) => {
                if cp.default.is_none() {
                    seen_non_default_or_const = true;
                }
            }
            syn::GenericParam::Lifetime(_) => {}
        }
    }
    if let Some(wc) = &mut mock_struct_generics.where_clause {
        rewrite_assoc_in_where(wc, &assoc_map);
    }

    let has_mock_generics = !mock_struct_generics.params.is_empty();
    let struct_param_idents: Vec<syn::Ident> = mock_struct_generics
        .params
        .iter()
        .filter_map(|p| match p {
            syn::GenericParam::Type(tp) => Some(tp.ident.clone()),
            _ => None,
        })
        .collect();
    let mut fields = Vec::new();
    let mut clone_fields = Vec::new();
    let mut constructors = Vec::new();
    let mut assoc_impls = Vec::new();
    let mut methods = Vec::new();
    let mut checks = Vec::new();
    let mut non_generic_mock_bounds: Vec<syn::WherePredicate> = Vec::new();
    let mut to_owned_bounds: Vec<syn::WherePredicate> = Vec::new();

    for info in &assoc_items {
        let assoc_ident = &info.assoc_ident;
        let param_ident = &info.param_ident;
        assoc_impls.push(quote!(type #assoc_ident = #param_ident;));
    }

    for item in &input.items {
        let syn::TraitItem::Fn(method) = item else {
            continue;
        };
        let sig = &method.sig;
        if sig.unsafety.is_some() || sig.abi.is_some() || sig.variadic.is_some() {
            return Err(syn::Error::new_spanned(
                sig,
                "mock methods must be safe Rust methods without extern ABI or variadics; \
                 use a manual Mock with mock::borrowed / ExpectationBuilder::map_args",
            ));
        }
        let is_generic_method = !sig.generics.params.is_empty();
        if let Some(lt) = sig
            .generics
            .params
            .iter()
            .find(|p| matches!(p, syn::GenericParam::Lifetime(_)))
        {
            return Err(syn::Error::new_spanned(
                lt,
                "generic mock methods with lifetime parameters require a manual mock adapter",
            ));
        }
        if is_generic_method {
            for param in &sig.generics.params {
                if let syn::GenericParam::Type(tp) = param
                    && !type_has_static_bound(&tp.bounds)
                    && !where_has_static_bound(sig.generics.where_clause.as_ref(), &tp.ident)
                    && !where_has_static_bound(input.generics.where_clause.as_ref(), &tp.ident)
                {
                    return Err(syn::Error::new_spanned(
                        tp,
                        "generic mock methods require 'static bounds on method type parameters \
                         (for example fn get<T: 'static>(&self, ...))",
                    ));
                }
            }
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
        let mut result: syn::Type = match &sig.output {
            syn::ReturnType::Default => parse_quote!(()),
            syn::ReturnType::Type(_, ty) => (**ty).clone(),
        };
        rewrite_assoc_in_type(&mut result, &assoc_map);

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
            let mut normalized_ty = (*arg.ty).clone();
            rewrite_assoc_in_type(&mut normalized_ty, &assoc_map);

            let (owned, value) = if let syn::Type::Reference(reference) = &normalized_ty {
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
                if struct_param_idents
                    .iter()
                    .any(|ident| contains_ident(ty, ident))
                {
                    to_owned_bounds.push(parse_quote!(#ty: ::std::borrow::ToOwned));
                }
                (
                    quote!(<#ty as ::std::borrow::ToOwned>::Owned),
                    quote!(::std::borrow::ToOwned::to_owned(#local)),
                )
            } else {
                let mut unsupported = Unsupported(false);
                unsupported.visit_type(&normalized_ty);
                if unsupported.0 {
                    return Err(syn::Error::new_spanned(
                        &arg.ty,
                        "nested references and Self arguments require a manual mock adapter",
                    ));
                }
                (quote!(#normalized_ty), quote!(#local))
            };
            arg_types.push(owned);
            arg_values.push(value);
        }

        let mut phantom_method_params = Vec::new();
        if is_generic_method {
            for param in &sig.generics.params {
                if let syn::GenericParam::Type(tp) = param {
                    let mentioned_in_ret = contains_ident(&result, &tp.ident);
                    let mentioned_in_args = impl_sig.inputs.iter().skip(1).any(|arg| match arg {
                        syn::FnArg::Typed(t) => contains_ident(&t.ty, &tp.ident),
                        _ => false,
                    });
                    if !mentioned_in_ret && !mentioned_in_args {
                        let ident = &tp.ident;
                        phantom_method_params.push((
                            quote!(::core::marker::PhantomData<#ident>),
                            quote!(::core::marker::PhantomData),
                        ));
                    }
                }
            }
        }

        if is_generic_method {
            let mut key_arg_types = arg_types.clone();
            let mut key_arg_values = arg_values.clone();
            for (pt, pv) in phantom_method_params {
                key_arg_types.push(pt);
                key_arg_values.push(pv);
            }
            fields.push(quote!(pub #id: #root::mock::GenericMock));
            clone_fields.push(quote!(#id: self.#id.clone()));
            constructors.push(quote!(#id: #root::mock::GenericMock::new(concat!(stringify!(#name), "::", stringify!(#id)))));
            methods.push(quote!(
                #impl_sig {
                    self.#id.call::<(#(#key_arg_types,)*), #result>((#(#key_arg_values,)*))
                }
            ));
            checks.push(quote!(if let ::core::result::Result::Err(error) = self.#id.verify() { errors.push(error); }));
        } else {
            let tuple_ty: syn::Type = parse_quote!((#(#arg_types,)*));
            if has_mock_generics {
                non_generic_mock_bounds.push(parse_quote!(#tuple_ty: ::core::fmt::Debug + 'static));
                non_generic_mock_bounds.push(parse_quote!(#result: 'static));
            }
            fields.push(quote!(pub #id: #root::Mock<#tuple_ty, #result>));
            clone_fields.push(quote!(#id: self.#id.clone()));
            constructors.push(
                quote!(#id: #root::Mock::new(concat!(stringify!(#name), "::", stringify!(#id)))),
            );
            methods.push(quote!(#impl_sig { self.#id.call((#(#arg_values,)*)) }));
            checks.push(quote!(if let ::core::result::Result::Err(error) = self.#id.verify() { errors.push(error); }));
        }
    }

    let mut phantom_field_def = None;
    let mut phantom_field_init = None;
    let mut phantom_field_clone = None;
    if has_mock_generics {
        let mut phantom_elems = Vec::new();
        for p in &mock_struct_generics.params {
            if let syn::GenericParam::Type(tp) = p {
                let id = &tp.ident;
                phantom_elems.push(quote!(#id));
            }
        }
        phantom_field_def = Some(
            quote!(_airbug_phantom: ::core::marker::PhantomData<fn() -> (#(#phantom_elems,)*)>),
        );
        phantom_field_init = Some(quote!(_airbug_phantom: ::core::marker::PhantomData));
        phantom_field_clone = Some(quote!(_airbug_phantom: ::core::marker::PhantomData));
    }

    if !to_owned_bounds.is_empty() {
        let wc = mock_struct_generics.make_where_clause();
        for pred in &to_owned_bounds {
            if !wc
                .predicates
                .iter()
                .any(|existing| quote!(#existing).to_string() == quote!(#pred).to_string())
            {
                wc.predicates.push(pred.clone());
            }
        }
    }

    let (_, trait_ty_generics, trait_where) = input.generics.split_for_impl();
    let (mock_impl_generics, mock_ty_generics, mock_where) = mock_struct_generics.split_for_impl();

    let mut impl_with_mock_bounds_generics = mock_struct_generics.clone();
    if !non_generic_mock_bounds.is_empty() {
        let wc = impl_with_mock_bounds_generics.make_where_clause();
        for pred in non_generic_mock_bounds {
            wc.predicates.push(pred);
        }
    }
    let (_, _, impl_where_with_bounds) = impl_with_mock_bounds_generics.split_for_impl();

    let mut trait_impl_generics = mock_struct_generics.clone();
    trait_impl_generics.where_clause = trait_where.cloned();
    if let Some(extra_wc) = &impl_with_mock_bounds_generics.where_clause {
        let wc = trait_impl_generics.make_where_clause();
        for pred in &extra_wc.predicates {
            // Avoid duplicating identical predicates already present in trait_where
            if !wc
                .predicates
                .iter()
                .any(|existing| quote!(#existing).to_string() == quote!(#pred).to_string())
            {
                wc.predicates.push(pred.clone());
            }
        }
    }
    let (_, _, trait_impl_where) = trait_impl_generics.split_for_impl();

    let cfg: Vec<_> = input
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("cfg") || a.path().is_ident("cfg_attr"))
        .collect();

    Ok(quote! {
        #input
        #(#cfg)*
        #vis struct #mock_name #mock_struct_generics #mock_where {
            #(#fields,)*
            #phantom_field_def
        }
        #(#cfg)*
        impl #mock_impl_generics ::core::clone::Clone for #mock_name #mock_ty_generics #mock_where {
            fn clone(&self) -> Self {
                Self {
                    #(#clone_fields,)*
                    #phantom_field_clone
                }
            }
        }
        #(#cfg)*
        impl #mock_impl_generics ::core::default::Default for #mock_name #mock_ty_generics #impl_where_with_bounds {
            fn default() -> Self {
                Self {
                    #(#constructors,)*
                    #phantom_field_init
                }
            }
        }
        #(#cfg)*
        impl #mock_impl_generics #name #trait_ty_generics for #mock_name #mock_ty_generics #trait_impl_where {
            #(#assoc_impls)*
            #(#methods)*
        }
        #(#cfg)*
        impl #mock_impl_generics #root::mock::VerifyMocks for #mock_name #mock_ty_generics #impl_where_with_bounds {
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

struct TestMetaArgs {
    epic: Option<syn::LitStr>,
    feature: Option<syn::LitStr>,
    story: Option<syn::LitStr>,
    severity: Option<syn::LitStr>,
    owner: Option<syn::LitStr>,
    tms: Option<syn::LitStr>,
    issue: Option<syn::LitStr>,
    tags: Vec<syn::LitStr>,
    quarantined: Option<syn::LitStr>,
    flaky: Option<syn::LitStr>,
}

impl TestMetaArgs {
    fn parse(attr: TokenStream) -> syn::Result<Self> {
        let mut out = Self {
            epic: None,
            feature: None,
            story: None,
            severity: None,
            owner: None,
            tms: None,
            issue: None,
            tags: Vec::new(),
            quarantined: None,
            flaky: None,
        };
        if attr.is_empty() {
            return Ok(out);
        }
        let parser = syn::meta::parser(|meta| {
            if meta.path.is_ident("epic") {
                out.epic = Some(meta.value()?.parse()?);
                Ok(())
            } else if meta.path.is_ident("feature") {
                out.feature = Some(meta.value()?.parse()?);
                Ok(())
            } else if meta.path.is_ident("story") {
                out.story = Some(meta.value()?.parse()?);
                Ok(())
            } else if meta.path.is_ident("severity") {
                let s: syn::LitStr = meta.value()?.parse()?;
                let val = s.value();
                if !matches!(
                    val.as_str(),
                    "blocker" | "critical" | "normal" | "minor" | "trivial"
                ) {
                    return Err(syn::Error::new_spanned(
                        &s,
                        "severity must be one of: blocker, critical, normal, minor, trivial",
                    ));
                }
                out.severity = Some(s);
                Ok(())
            } else if meta.path.is_ident("owner") {
                out.owner = Some(meta.value()?.parse()?);
                Ok(())
            } else if meta.path.is_ident("tms") {
                out.tms = Some(meta.value()?.parse()?);
                Ok(())
            } else if meta.path.is_ident("issue") {
                out.issue = Some(meta.value()?.parse()?);
                Ok(())
            } else if meta.path.is_ident("tag") {
                out.tags.push(meta.value()?.parse()?);
                Ok(())
            } else if meta.path.is_ident("tags") {
                let expr: syn::ExprArray = meta.value()?.parse()?;
                for elem in expr.elems {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = elem
                    {
                        out.tags.push(s);
                    } else {
                        return Err(syn::Error::new_spanned(
                            elem,
                            "tags array elements must be string literals",
                        ));
                    }
                }
                Ok(())
            } else if meta.path.is_ident("quarantined") {
                out.quarantined = Some(meta.value()?.parse()?);
                Ok(())
            } else if meta.path.is_ident("flaky") {
                out.flaky = Some(meta.value()?.parse()?);
                Ok(())
            } else {
                Err(meta.error(
                    "unsupported #[airbug::test] argument; expected epic, feature, story, severity, owner, tms, issue, tag, tags, quarantined, or flaky",
                ))
            }
        });
        syn::parse::Parser::parse(parser, attr)?;
        Ok(out)
    }
}

/// Annotate a test with TestOps taxonomy (`epic`, `feature`, `story`, `severity`, `owner`, `tms`,
/// `issue`, `tag` / `tags`, `quarantined`, `flaky`). Emits `#[test]` on synchronous functions when
/// no test attribute is already present, and preserves existing test runners such as `#[tokio::test]`.
#[proc_macro_attribute]
pub fn test(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args = match TestMetaArgs::parse(attr) {
        Ok(args) => args,
        Err(err) => return err.into_compile_error().into(),
    };
    finish(expand_test(args, parse_macro_input!(input as syn::ItemFn)))
}

fn has_test_attribute(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "test")
    })
}

fn expand_test(args: TestMetaArgs, mut input: syn::ItemFn) -> syn::Result<Tokens> {
    let root = runtime()?;
    let mut stmts: Vec<syn::Stmt> = Vec::new();
    if let Some(v) = args.epic {
        stmts.push(parse_quote!(#root::report::epic(#v);));
    }
    if let Some(v) = args.feature {
        stmts.push(parse_quote!(#root::report::feature(#v);));
    }
    if let Some(v) = args.story {
        stmts.push(parse_quote!(#root::report::story(#v);));
    }
    if let Some(v) = args.severity {
        stmts.push(parse_quote!(#root::report::severity(#v);));
    }
    if let Some(v) = args.owner {
        stmts.push(parse_quote!(#root::report::owner(#v);));
    }
    for tag in args.tags {
        stmts.push(parse_quote!(#root::report::tag(#tag);));
    }
    if let Some(v) = args.tms {
        stmts.push(parse_quote!(#root::report::tms_link(#v);));
    }
    if let Some(v) = args.issue {
        stmts.push(parse_quote!(#root::report::issue_link(#v);));
    }
    if let Some(v) = args.quarantined {
        stmts.push(parse_quote!(#root::report::quarantine(#v);));
    }
    if let Some(v) = args.flaky {
        stmts.push(parse_quote!(#root::report::flaky(#v);));
    }

    let old_stmts = std::mem::take(&mut input.block.stmts);
    input.block.stmts = stmts;
    input.block.stmts.extend(old_stmts);

    let needs_std_test = input.sig.asyncness.is_none() && !has_test_attribute(&input.attrs);
    if needs_std_test {
        Ok(quote! {
            #[::core::prelude::v1::test]
            #input
        })
    } else {
        Ok(quote!(#input))
    }
}
