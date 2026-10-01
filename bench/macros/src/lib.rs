use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Expr, ItemFn, Meta, Token, Type,
    ext::IdentExt,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

#[derive(Clone, Default)]
struct Options {
    args: Option<Expr>,
    crate_path: Option<syn::Path>,
    groups: Option<Vec<syn::Path>>,
    setup: Option<Expr>,
    setup_thread: Option<syn::LitStr>,
    executor: Option<Expr>,
    allocator: Option<Expr>,
    measurement: Option<Expr>,
    formatter: Option<Expr>,
    timer: Option<syn::LitStr>,
    summary_scale: Option<syn::LitStr>,
    summary_family: Option<syn::LitStr>,
    overhead: Option<syn::LitStr>,
    significance_level: Option<Expr>,
    noise_threshold_percent: Option<Expr>,
    hypothesis_resamples: Option<Expr>,
    hypothesis_seed: Option<Expr>,
    resamples: Option<Expr>,
    confidence_level: Option<Expr>,
    analysis_seed: Option<Expr>,
    quick: Option<syn::LitBool>,
    quick_config: Option<Expr>,
    threads: Option<Expr>,
    inherited_thread_dispatch: bool,
    bytes: Option<Expr>,
    items: Option<Expr>,
    chars: Option<Expr>,
    cycles: Option<Expr>,
    bits: Option<Expr>,
    input_bytes: Option<Expr>,
    input_items: Option<Expr>,
    input_chars: Option<Expr>,
    input_cycles: Option<Expr>,
    input_bits: Option<Expr>,
    ignore: Option<Expr>,
    samples: Option<Expr>,
    sample_count_unit: Option<syn::LitStr>,
    worker_start: Option<syn::LitStr>,
    warmup_ms: Option<Expr>,
    sample_ms: Option<Expr>,
    measurement_ms: Option<Expr>,
    iterations: Option<Expr>,
    sampling: Option<syn::LitStr>,
    min_time_ms: Option<Expr>,
    max_time_ms: Option<Expr>,
    exclude_external_time: Option<syn::LitBool>,
    types: Option<Vec<Type>>,
    consts: Option<Expr>,
    custom: bool,
    batch: Option<Expr>,
    name: Option<syn::LitStr>,
    drop_output: Option<syn::LitStr>,
}

impl Parse for Options {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut options = Self::default();
        let mut seen = std::collections::BTreeSet::new();
        while !input.is_empty() {
            let key: syn::Ident = input.call(syn::Ident::parse_any)?;
            if !seen.insert(key.to_string()) {
                return Err(syn::Error::new_spanned(key, "duplicate benchmark option"));
            }
            if key == "threads" && (input.is_empty() || input.peek(Token![,])) {
                options.threads = Some(syn::parse_quote!(true));
                if input.is_empty() {
                    break;
                }
                input.parse::<Token![,]>()?;
                continue;
            }
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "significance_level" => options.significance_level = Some(input.parse()?),
                "noise_threshold_percent" => options.noise_threshold_percent = Some(input.parse()?),
                "hypothesis_resamples" => options.hypothesis_resamples = Some(input.parse()?),
                "hypothesis_seed" => options.hypothesis_seed = Some(input.parse()?),
                "resamples" => options.resamples = Some(input.parse()?),
                "confidence_level" => options.confidence_level = Some(input.parse()?),
                "analysis_seed" => options.analysis_seed = Some(input.parse()?),
                "overhead" => {
                    let value: syn::LitStr = input.parse()?;
                    if !matches!(value.value().as_str(), "raw" | "subtract") {
                        return Err(syn::Error::new_spanned(
                            value,
                            "overhead must be raw or subtract",
                        ));
                    }
                    options.overhead = Some(value);
                }
                "summary_family" => options.summary_family = Some(input.parse()?),
                "summary_scale" => {
                    let value: syn::LitStr = input.parse()?;
                    if !matches!(value.value().as_str(), "linear" | "logarithmic") {
                        return Err(syn::Error::new_spanned(
                            value,
                            "summary_scale must be linear or logarithmic",
                        ));
                    }
                    options.summary_scale = Some(value);
                }
                "timer" => {
                    let value: syn::LitStr = input.parse()?;
                    if !matches!(value.value().as_str(), "os" | "cpu") {
                        return Err(syn::Error::new_spanned(value, "timer must be os or cpu"));
                    }
                    options.timer = Some(value);
                }
                "quick" => options.quick = Some(input.parse()?),
                "quick_config" => options.quick_config = Some(input.parse()?),
                "measurement" => options.measurement = Some(input.parse()?),
                "formatter" => options.formatter = Some(input.parse()?),
                "allocator" => options.allocator = Some(input.parse()?),
                "batch" => options.batch = Some(input.parse()?),
                "crate" => options.crate_path = Some(input.parse()?),
                "groups" => {
                    let content;
                    syn::bracketed!(content in input);
                    options.groups = Some(
                        Punctuated::<syn::Path, Token![,]>::parse_terminated(&content)?
                            .into_iter()
                            .collect(),
                    );
                }
                "types" => {
                    let content;
                    syn::bracketed!(content in input);
                    let values = Punctuated::<Type, Token![,]>::parse_terminated(&content)?;
                    if values.is_empty() {
                        return Err(content.error("types must not be empty"));
                    }
                    options.types = Some(values.into_iter().collect());
                }
                "consts" => {
                    let value: Expr = input.parse()?;
                    if matches!(&value, Expr::Array(array) if array.elems.is_empty()) {
                        return Err(syn::Error::new_spanned(value, "consts must not be empty"));
                    }
                    options.consts = Some(value);
                }
                "name" => options.name = Some(input.parse()?),
                "drop_output" => {
                    let value: syn::LitStr = input.parse()?;
                    if !matches!(value.value().as_str(), "inside" | "outside") {
                        return Err(syn::Error::new_spanned(
                            value,
                            "drop_output must be \"inside\" or \"outside\"",
                        ));
                    }
                    options.drop_output = Some(value);
                }
                "custom" => options.custom = input.parse::<syn::LitBool>()?.value,
                "args" => options.args = Some(input.parse()?),
                "setup" => options.setup = Some(input.parse()?),
                "setup_thread" => {
                    let value: syn::LitStr = input.parse()?;
                    if !matches!(value.value().as_str(), "worker" | "coordinator") {
                        return Err(syn::Error::new_spanned(
                            value,
                            "setup_thread must be worker or coordinator",
                        ));
                    }
                    options.setup_thread = Some(value);
                }
                "executor" => options.executor = Some(input.parse()?),
                "threads" => options.threads = Some(input.parse()?),
                "bytes" => options.bytes = Some(input.parse()?),
                "items" => options.items = Some(input.parse()?),
                "chars" => options.chars = Some(input.parse()?),
                "cycles" => options.cycles = Some(input.parse()?),
                "bits" => options.bits = Some(input.parse()?),
                "input_bytes" => options.input_bytes = Some(input.parse()?),
                "input_items" => options.input_items = Some(input.parse()?),
                "input_chars" => options.input_chars = Some(input.parse()?),
                "input_cycles" => options.input_cycles = Some(input.parse()?),
                "input_bits" => options.input_bits = Some(input.parse()?),
                "ignore" => options.ignore = Some(input.parse()?),
                "iterations" => options.iterations = Some(input.parse()?),
                "min_time_ms" => options.min_time_ms = Some(input.parse()?),
                "max_time_ms" => options.max_time_ms = Some(input.parse()?),
                "exclude_external_time" => options.exclude_external_time = Some(input.parse()?),
                "sampling" => {
                    let value: syn::LitStr = input.parse()?;
                    if !matches!(value.value().as_str(), "flat" | "linear" | "auto") {
                        return Err(syn::Error::new_spanned(
                            value,
                            "sampling must be flat, linear or auto",
                        ));
                    }
                    options.sampling = Some(value);
                }
                "worker_start" => {
                    let value: syn::LitStr = input.parse()?;
                    if !matches!(value.value().as_str(), "shared" | "local") {
                        return Err(syn::Error::new(
                            value.span(),
                            "worker_start must be shared or local",
                        ));
                    }
                    options.worker_start = Some(value);
                }
                "sample_count_unit" => {
                    let value: syn::LitStr = input.parse()?;
                    if !matches!(value.value().as_str(), "batches" | "workers") {
                        return Err(syn::Error::new(
                            value.span(),
                            "sample_count_unit must be batches or workers",
                        ));
                    }
                    options.sample_count_unit = Some(value);
                }
                "samples" => options.samples = Some(input.parse()?),
                "warmup_ms" => options.warmup_ms = Some(input.parse()?),
                "sample_ms" => options.sample_ms = Some(input.parse()?),
                "measurement_ms" => options.measurement_ms = Some(input.parse()?),
                _ => return Err(syn::Error::new_spanned(key, "unknown benchmark option")),
            }
            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }
        Ok(options)
    }
}

fn runtime_path(options: &Options) -> syn::Result<syn::Path> {
    if let Some(path) = &options.crate_path {
        return Ok(path.clone());
    }
    match proc_macro_crate::crate_name("airbug-bench") {
        Ok(proc_macro_crate::FoundCrate::Itself) => Ok(syn::parse_quote!(::airbug_bench)),
        Ok(proc_macro_crate::FoundCrate::Name(name)) => {
            let ident = format_ident!("{}", name);
            Ok(syn::parse_quote!(::#ident))
        }
        Err(error) => Err(syn::Error::new(
            proc_macro::Span::call_site().into(),
            format!("cannot locate airbug-bench: {error}; set crate = path explicitly"),
        )),
    }
}

// Type arguments in the existing generic registrar are constrained to 'static.
// Preserve that meaning when using concrete aliases for lifetime-aware functions.
struct AliasLifetimes;
impl syn::visit_mut::VisitMut for AliasLifetimes {
    fn visit_type_reference_mut(&mut self, ty: &mut syn::TypeReference) {
        if ty.lifetime.as_ref().is_none_or(|l| l.ident == "_") {
            ty.lifetime = Some(syn::parse_quote!('static));
        }
        syn::visit_mut::visit_type_reference_mut(self, ty);
    }
    fn visit_lifetime_mut(&mut self, lifetime: &mut syn::Lifetime) {
        if lifetime.ident == "_" {
            *lifetime = syn::parse_quote!('static);
        }
    }
    // Bare function pointer elision is higher-ranked; it must stay that way.
    fn visit_type_bare_fn_mut(&mut self, _: &mut syn::TypeBareFn) {}
    fn visit_parenthesized_generic_arguments_mut(
        &mut self,
        _: &mut syn::ParenthesizedGenericArguments,
    ) {
    }
}

fn bootstrap_defaults(options: &Options, runtime: &syn::Path) -> impl quote::ToTokens {
    let optional = |value: &Option<Expr>| {
        value
            .as_ref()
            .map(|v| quote!(Some(#v)))
            .unwrap_or_else(|| quote!(None))
    };
    let resamples = optional(&options.resamples);
    let confidence = optional(&options.confidence_level);
    let seed = optional(&options.analysis_seed);
    quote!(#runtime::bootstrap::Options { resamples: #resamples, confidence_level: #confidence, seed: #seed })
}

fn comparison_defaults(options: &Options, runtime: &syn::Path) -> impl quote::ToTokens {
    let optional = |value: &Option<Expr>| {
        value
            .as_ref()
            .map(|v| quote!(Some(#v)))
            .unwrap_or_else(|| quote!(None))
    };
    let significance = optional(&options.significance_level);
    let noise = optional(&options.noise_threshold_percent);
    let resamples = optional(&options.hypothesis_resamples);
    let seed = optional(&options.hypothesis_seed);
    quote!(#runtime::history::ComparisonOptions {
        significance_level: #significance,
        noise_threshold_percent: #noise,
        resamples: #resamples,
        seed: #seed,
    })
}
fn summary_scale_default(options: &Options, runtime: &syn::Path) -> Option<impl quote::ToTokens> {
    options.summary_scale.as_ref().map(|value| {
        let variant = if value.value() == "linear" {
            format_ident!("Linear")
        } else {
            format_ident!("Logarithmic")
        };
        quote!(#runtime::viz::charts::AxisScale::#variant)
    })
}

fn timer_defaults(options: &Options, runtime: &syn::Path) -> Option<impl quote::ToTokens> {
    options.timer.as_ref().map(|value| {
        let variant = if value.value() == "cpu" {
            quote!(Cpu)
        } else {
            quote!(Os)
        };
        quote!(#runtime::timer::TimerKind::#variant)
    })
}

fn quick_defaults(options: &Options, runtime: &syn::Path) -> Option<impl quote::ToTokens> {
    if options.quick.is_none() && options.quick_config.is_none() {
        return None;
    }
    let enabled = options.quick.as_ref().map(|b| b.value).unwrap_or(true);
    let config = options
        .quick_config
        .as_ref()
        .map(|c| quote!(#c))
        .unwrap_or_else(|| quote!(#runtime::QuickConfig::default()));
    Some(if enabled {
        quote!(Some(#config))
    } else {
        quote!(None)
    })
}

fn attribute_milliseconds(value: &Expr, runtime: &syn::Path) -> impl quote::ToTokens {
    fn integer_expression(value: &Expr) -> bool {
        match value {
            Expr::Lit(expr) => matches!(&expr.lit, syn::Lit::Int(lit) if lit.suffix().is_empty()),
            Expr::Binary(expr) => integer_expression(&expr.left) && integer_expression(&expr.right),
            Expr::Paren(expr) => integer_expression(&expr.expr),
            Expr::Group(expr) => integer_expression(&expr.expr),
            _ => false,
        }
    }
    // Preserve the former u64 context for untyped integer arithmetic as well as
    // literals. Other expressions retain their own type before decimal conversion.
    let value = if integer_expression(value) {
        quote!({ let value: u64 = #value; value })
    } else {
        quote!(#value)
    };
    quote!(#runtime::__milliseconds(&::std::string::ToString::to_string(&(#value)))
        .expect("valid benchmark millisecond duration"))
}

fn sampling_defaults(options: &Options, runtime: &syn::Path) -> impl quote::ToTokens {
    let samples = options
        .samples
        .as_ref()
        .map(|e| quote!(Some(#e)))
        .unwrap_or_else(|| quote!(None));
    let warmup = options
        .warmup_ms
        .as_ref()
        .map(|e| {
            let duration = attribute_milliseconds(e, runtime);
            quote!(Some(#duration))
        })
        .unwrap_or_else(|| quote!(None));
    let sample = options
        .sample_ms
        .as_ref()
        .map(|e| {
            let duration = attribute_milliseconds(e, runtime);
            quote!(Some(#duration))
        })
        .unwrap_or_else(|| quote!(None));
    let iterations = options
        .iterations
        .as_ref()
        .map(|e| quote!(Some(#e)))
        .unwrap_or_else(|| quote!(None));
    let measurement = options
        .measurement_ms
        .as_ref()
        .map(|e| {
            let duration = attribute_milliseconds(e, runtime);
            quote!(Some(#duration))
        })
        .unwrap_or_else(|| quote!(None));
    let mode = options
        .sampling
        .as_ref()
        .map(|e| {
            let variant = match e.value().as_str() {
                "linear" => quote!(Linear),
                "auto" => quote!(Auto),
                _ => quote!(Flat),
            };
            quote!(Some(#runtime::SamplingMode::#variant))
        })
        .unwrap_or_else(|| quote!(None));
    let duration = |value: &Option<Expr>| {
        value
            .as_ref()
            .map(|e| {
                let duration = attribute_milliseconds(e, runtime);
                quote!(Some(#duration))
            })
            .unwrap_or_else(|| quote!(None))
    };
    let min_time = duration(&options.min_time_ms);
    let max_time = duration(&options.max_time_ms);
    let exclude_external_time = options
        .exclude_external_time
        .as_ref()
        .map(|e| quote!(Some(#e)))
        .unwrap_or_else(|| quote!(None));
    let sample_count_unit = options
        .sample_count_unit
        .as_ref()
        .map(|s| {
            let variant = if s.value() == "workers" {
                quote!(Workers)
            } else {
                quote!(Batches)
            };
            quote!(Some(#runtime::SampleCountUnit::#variant))
        })
        .unwrap_or_else(|| quote!(None));
    let worker_start = options
        .worker_start
        .as_ref()
        .map(|s| {
            let variant = if s.value() == "local" {
                quote!(Local)
            } else {
                quote!(Shared)
            };
            quote!(Some(#runtime::timer::WorkerStart::#variant))
        })
        .unwrap_or_else(|| quote!(None));
    quote!(#runtime::Sampling { worker_start: #worker_start, sample_count_unit: #sample_count_unit, samples: #samples, warmup: #warmup, sample_time: #sample, measurement_time: #measurement, iterations: #iterations, mode: #mode, min_time: #min_time, max_time: #max_time, exclude_external_time: #exclude_external_time })
}

fn registration(f: &ItemFn, mut options: Options) -> syn::Result<syn::Block> {
    if options.threads.is_some() {
        let disabled = matches!(&options.threads, Some(Expr::Lit(expr)) if matches!(&expr.lit, syn::Lit::Bool(value) if !value.value));
        let local_dispatch = !disabled
            && !options.custom
            && options.measurement.is_none()
            && options.setup_thread.is_none()
            && (f.sig.asyncness.is_none()
                || [
                    &options.input_bytes,
                    &options.input_items,
                    &options.input_chars,
                    &options.input_cycles,
                    &options.input_bits,
                ]
                .into_iter()
                .flatten()
                .all(|expr| matches!(expr, Expr::Closure(_) | Expr::Path(_))));
        options.inherited_thread_dispatch = local_dispatch;
        return registration_batch(f, options);
    }
    let original = registration_batch(f, options.clone())?;
    // Constructed async counters use the local fallback under inherited defaults;
    // concrete threads remain available for explicitly shareable counter state.
    if !options.custom && options.measurement.is_none() {
        let mut threaded = options;
        threaded.inherited_thread_dispatch = true;
        threaded.threads = Some(syn::parse_quote!(
            suite
                .inherited_thread_counts()
                .expect("inherited worker counts")
        ));
        let threaded = registration_batch(f, threaded)?;
        return syn::parse2(quote!({
            if suite.inherited_thread_counts().is_some() {
                #threaded
            } else {
                #original
            }
        }));
    }
    syn::parse2(quote!({
        suite.register_without_thread_support(|suite| #original);
    }))
}

fn registration_batch(f: &ItemFn, mut options: Options) -> syn::Result<syn::Block> {
    if matches!(&options.threads, Some(Expr::Lit(expr)) if matches!(&expr.lit, syn::Lit::Bool(value) if !value.value))
    {
        options.threads = None;
        options.setup_thread = None;
    }
    // Threaded executors read the scoped batch policy at registration. Keep
    // batch out of the sequential method selector below.
    if options.threads.is_some() && !options.custom {
        let batch = options.batch.take();
        let original = registration_impl(f, options)?;
        return if let Some(batch) = batch {
            syn::parse2(quote!({ suite.with_batch_defaults(#batch, |suite| #original); }))
        } else {
            Ok(original)
        };
    }
    let original = registration_impl(f, options.clone())?;
    if options.batch.is_some() {
        return Ok(original);
    }
    if options.custom || options.threads.is_some() {
        return syn::parse2(quote!({
            assert!(!suite.has_registration_batch_policy(), "imported batch cannot combine with threads or custom timing");
            #original
        }));
    }
    let mut inherited = options;
    inherited.batch = Some(syn::parse_quote!(suite.registration_batch_policy()));
    let batched = registration_impl(f, inherited)?;
    syn::parse2(quote!({
        if suite.has_registration_batch_policy() { #batched } else { #original }
    }))
}

fn registration_impl(f: &ItemFn, mut options: Options) -> syn::Result<syn::Block> {
    let runtime = runtime_path(&options)?;
    if options.groups.is_some() {
        return Err(syn::Error::new_spanned(
            &f.sig,
            "groups is a suite/group option",
        ));
    }
    // A literal false disables inherited parallelism before selecting a lifecycle.
    if matches!(&options.threads, Some(Expr::Lit(expr)) if matches!(&expr.lit, syn::Lit::Bool(value) if !value.value))
    {
        options.threads = None;
        options.setup_thread = None;
    }
    let expected_inputs = usize::from(options.args.is_some() || options.setup.is_some())
        + usize::from(options.custom);
    if f.sig.inputs.len() != expected_inputs || f.sig.unsafety.is_some() {
        return Err(syn::Error::new_spanned(
            &f.sig,
            "bench requires a safe function: one input with args/setup, plus an iteration count first with custom = true",
        ));
    }
    let is_async = f.sig.asyncness.is_some();
    if options.executor.is_some() && !is_async {
        return Err(syn::Error::new_spanned(
            &f.sig,
            "executor requires an async function",
        ));
    }
    if options.setup_thread.is_some() && options.setup.is_none() {
        return Err(syn::Error::new_spanned(
            &f.sig,
            "setup_thread requires setup",
        ));
    }
    let input_counts: Vec<_> = [
        ("bytes", &options.input_bytes),
        ("items", &options.input_items),
        ("chars", &options.input_chars),
        ("cycles", &options.input_cycles),
        ("bits", &options.input_bits),
    ]
    .into_iter()
    .filter_map(|(unit, expr)| expr.as_ref().map(|expr| (unit, expr)))
    .collect();
    if !input_counts.is_empty() && options.setup.is_none() {
        return Err(syn::Error::new_spanned(
            &f.sig,
            "input counters require setup",
        ));
    }
    let units: Vec<_> = input_counts.iter().map(|(unit, _)| unit).collect();
    let counter_bound =
        if (is_async && options.threads.is_some() && !options.inherited_thread_dispatch)
            || options
                .setup_thread
                .as_ref()
                .is_some_and(|v| v.value() == "worker")
        {
            quote!(Fn)
        } else {
            quote!(FnMut)
        };
    let counter_functions: Vec<_> = input_counts
        .iter()
        .map(|(unit, expr)| {
            let name = format_ident!("__airbug_input_count_{unit}");
            if is_async && options.inherited_thread_dispatch {
                quote!(let #name = ::std::sync::Mutex::new(__airbug_input_counter(#expr));)
            } else {
                quote!(#[allow(unused_mut)] let mut #name = __airbug_input_counter(#expr);)
            }
        })
        .collect();
    let input_counter_init = (!units.is_empty()).then(|| quote! {
        fn __airbug_input_counter<I: ?Sized, F: #counter_bound(&I) -> u64>(counter: F) -> F { counter }
        let __airbug_input_counters = #runtime::counters::InputCounters::new(&[#(#units),*]).expect("valid units");
        let __airbug_setup_counters = __airbug_input_counters.clone();
        #(#counter_functions)*
    });
    let input_counter_attach =
        (!units.is_empty()).then(|| quote!(suite.input_counters(__airbug_input_counters);));
    let worker_setup = options
        .setup_thread
        .as_ref()
        .is_some_and(|v| v.value() == "worker");
    if options.batch.is_some() && (options.threads.is_some() || options.custom) {
        return Err(syn::Error::new_spanned(
            &f.sig,
            "batch cannot combine with threads or custom timing",
        ));
    }
    if options.custom
        && (options.setup.is_some() || options.threads.is_some() || options.drop_output.is_some())
    {
        return Err(syn::Error::new_spanned(
            &f.sig,
            "custom timing owns setup, drop and scheduling; it cannot combine with setup, threads or drop_output",
        ));
    }
    if is_async
        && options.threads.is_some()
        && options
            .setup_thread
            .as_ref()
            .is_some_and(|v| v.value() == "coordinator")
    {
        return Err(syn::Error::new_spanned(
            &f.sig,
            "async threads use worker-local setup; omit setup_thread or select worker",
        ));
    }

    let type_params: Vec<_> = f.sig.generics.type_params().collect();
    let const_params: Vec<_> = f.sig.generics.const_params().collect();
    if type_params.is_empty() == options.types.is_some()
        || const_params.is_empty() == options.consts.is_some()
    {
        return Err(syn::Error::new_spanned(
            &f.sig.generics,
            "supply types = [...] for type parameters and consts = [...] for const parameters",
        ));
    }
    let ident = &f.sig.ident;
    let generic_names: Vec<_> = f
        .sig
        .generics
        .params
        .iter()
        .filter_map(|param| match param {
            syn::GenericParam::Type(p) => {
                let id = &p.ident;
                Some(quote!(#id))
            }
            syn::GenericParam::Const(p) => {
                let id = &p.ident;
                Some(quote!({ #id }))
            }
            syn::GenericParam::Lifetime(_) => None,
        })
        .collect();
    let target = if generic_names.is_empty() {
        quote!(#ident)
    } else {
        quote!(#ident::<#(#generic_names),*>)
    };
    let name = options
        .name
        .as_ref()
        .map(|s| quote!(#s))
        .unwrap_or_else(|| quote!(stringify!(#ident)));
    let outside = options
        .drop_output
        .as_ref()
        .is_some_and(|v| v.value() == "outside");
    let policy = if options.drop_output.is_none() {
        quote!(suite.registration_drop_policy())
    } else if outside {
        quote!(#runtime::DropPolicy::OutsideTiming)
    } else {
        quote!(#runtime::DropPolicy::InsideTiming)
    };
    let has_args = options.args.is_some();
    let thread_argument_marker = if has_args {
        quote!(#runtime::threads::argument_marker(&__airbug_arg))
    } else {
        quote!(#runtime::threads::argument_marker(&()))
    };
    let borrowed_arg = has_args
        && options.setup.is_none()
        && matches!(f.sig.inputs.last(),
        Some(syn::FnArg::Typed(arg)) if matches!(&*arg.ty, Type::Reference(r) if r.mutability.is_none()));
    let arg = if borrowed_arg {
        quote!(&__airbug_arg)
    } else {
        quote!(__airbug_arg)
    };
    let invocation = if has_args {
        quote!(#target(::std::hint::black_box(#arg)))
    } else {
        quote!(#target())
    };
    let executor = options
        .executor
        .as_ref()
        .map(|e| quote!(#e))
        .unwrap_or_else(|| quote!(#runtime::workloads::LocalExecutor));
    let owned_input = options.setup.is_some()
        && matches!(f.sig.inputs.first(),
        Some(syn::FnArg::Typed(arg)) if !matches!(&*arg.ty, Type::Reference(_)));
    if options.allocator.is_some() && options.custom {
        return Err(syn::Error::new_spanned(
            &f.sig,
            "allocator cannot yet combine with custom timing",
        ));
    }
    if options.formatter.is_some() && options.measurement.is_none() {
        return Err(syn::Error::new_spanned(
            &f.sig,
            "formatter requires measurement",
        ));
    }
    if options.measurement.is_some() && (options.allocator.is_some() || options.threads.is_some()) {
        return Err(syn::Error::new_spanned(
            &f.sig,
            "measurement cannot yet combine with allocator or threads",
        ));
    }
    let body = if let Some(measurement) = &options.measurement {
        let batch = options.batch.as_ref().map(|b| quote!(#b)).unwrap_or_else(
            || quote!(#runtime::BatchPolicy::Iterations(::std::num::NonZeroU64::new(64).unwrap())),
        );
        let setup_call = if let Some(setup) = &options.setup {
            let call = if has_args {
                if matches!(setup, Expr::Closure(closure) if closure.inputs.iter().any(|input| !matches!(input, syn::Pat::Type(_))))
                {
                    quote!({
                        fn __airbug_setup<A, I>(setup: impl FnOnce(A) -> I, value: A) -> I { setup(value) }
                        __airbug_setup(#setup, __airbug_arg)
                    })
                } else {
                    quote!({
                        fn __airbug_setup<A: Copy, V: ::std::borrow::Borrow<A>, I>(
                            setup: impl FnOnce(A) -> I, value: &V,
                        ) -> I { setup(*value.borrow()) }
                        __airbug_setup(#setup, &__airbug_arg)
                    })
                }
            } else {
                quote!((#setup)())
            };
            let record = input_counts.iter().map(|(unit, _)| {
                let name = format_ident!("__airbug_input_count_{unit}");
                let name = if is_async && options.inherited_thread_dispatch {
                    quote!((#name.lock().expect("input counter mutex poisoned")))
                } else {
                    quote!(#name)
                };
                quote!(__airbug_setup_counters.add(#unit, #name(&__airbug_input));)
            });
            quote!({ let __airbug_input = #call; #(#record)* __airbug_input })
        } else {
            quote!(())
        };
        let call = if options.setup.is_some() {
            quote!(#target(__airbug_input))
        } else {
            invocation.clone()
        };
        let method = format_ident!(
            "bench_{}measured_with_{}input",
            if is_async { "async_" } else { "" },
            if owned_input { "owned_" } else { "" }
        );
        let register = if options.custom {
            let call = if has_args {
                quote!(#target(__airbug_iterations, ::std::hint::black_box(#arg)))
            } else {
                quote!(#target(__airbug_iterations))
            };
            if is_async {
                quote!(suite.bench_async_measured_custom(&__airbug_name, __airbug_measurement,
                    move || #executor, async move |__airbug_iterations| #call.await))
            } else {
                quote!(suite.bench_measured_custom(&__airbug_name, __airbug_measurement,
                    move |__airbug_iterations| #call))
            }
        } else if is_async {
            quote!(suite.#method(&__airbug_name, __airbug_measurement, #batch,
                move || #executor, move || #setup_call, async move |__airbug_input| #call.await, #policy))
        } else {
            quote!(suite.#method(&__airbug_name, __airbug_measurement, #batch,
                move || #setup_call, move |__airbug_input| #call, #policy))
        };
        let formatting = options.formatter.as_ref().map(|formatter|
            quote!(suite.formatter(&__airbug_metric.id, #formatter).expect("valid benchmark formatter");));
        quote! {
            let __airbug_measurement = #measurement;
            let __airbug_metric = #runtime::measurement::Measurement::metric(&__airbug_measurement);
            #register.expect("valid benchmark measurement");
            #formatting
        }
    } else if let Some(allocator) = &options.allocator {
        if options.setup.is_some()
            || options.batch.is_some()
            || outside
            || options.drop_output.is_none()
            || options.threads.is_some()
        {
            let batch = options.batch.as_ref().map(|b| quote!(#b)).unwrap_or_else(||
                quote!(#runtime::BatchPolicy::Iterations(::std::num::NonZeroU64::new(64).unwrap())));
            let setup_call = if let Some(setup) = &options.setup {
                let call = if has_args {
                    if matches!(setup, Expr::Closure(closure) if closure.inputs.iter().any(|input| !matches!(input, syn::Pat::Type(_))))
                    {
                        quote!({
                            fn __airbug_setup<A, I>(setup: impl FnOnce(A) -> I, value: A) -> I { setup(value) }
                            __airbug_setup(#setup, __airbug_arg)
                        })
                    } else {
                        quote!({
                            fn __airbug_setup<A: Copy, V: ::std::borrow::Borrow<A>, I>(
                                setup: impl FnOnce(A) -> I, value: &V,
                            ) -> I { setup(*value.borrow()) }
                            __airbug_setup(#setup, &__airbug_arg)
                        })
                    }
                } else {
                    quote!((#setup)())
                };
                let record = input_counts.iter().map(|(unit, _)| {
                    let name = format_ident!("__airbug_input_count_{unit}");
                    let name = if is_async && options.inherited_thread_dispatch {
                        quote!((#name.lock().expect("input counter mutex poisoned")))
                    } else {
                        quote!(#name)
                    };
                    quote!(__airbug_setup_counters.add(#unit, #name(&__airbug_input));)
                });
                quote!({ let __airbug_input = #call; #(#record)* __airbug_input })
            } else {
                quote!(())
            };
            let call = if options.setup.is_some() {
                quote!(#target(__airbug_input))
            } else {
                invocation.clone()
            };
            let method = format_ident!(
                "bench_{}allocated_batched{}",
                if is_async { "async_" } else { "" },
                if owned_input { "" } else { "_ref" }
            );
            if options.threads.is_some()
                && is_async
                && options.setup.is_some()
                && options.inherited_thread_dispatch
            {
                let (constructor, registration_trait, registration_method) = if owned_input {
                    (
                        quote!(inherited_async_owned_setup),
                        quote!(RegisterInheritedAsyncAllocatedOwnedSetup),
                        quote!(register_inherited_async_allocated_owned_setup),
                    )
                } else {
                    (
                        quote!(inherited_async_setup),
                        quote!(RegisterInheritedAsyncAllocatedSetup),
                        quote!(register_inherited_async_allocated_setup),
                    )
                };
                quote!({
                    use #runtime::threads::#registration_trait as _;
                    (&&#runtime::threads::#constructor(#thread_argument_marker, move || #executor,
                        move || #setup_call, async move |__airbug_input| #call.await))
                        .#registration_method(suite, &__airbug_name, #allocator, __airbug_workers, #policy);
                })
            } else if options.threads.is_some()
                && !is_async
                && options.setup.is_some()
                && !worker_setup
                && options.inherited_thread_dispatch
            {
                let (constructor, registration_trait, registration_method) = if owned_input {
                    (
                        quote!(inherited_owned_setup),
                        quote!(RegisterInheritedAllocatedOwnedSetup),
                        quote!(register_inherited_allocated_owned_setup),
                    )
                } else {
                    (
                        quote!(inherited_setup),
                        quote!(RegisterInheritedAllocatedSetup),
                        quote!(register_inherited_allocated_setup),
                    )
                };
                quote!({
                    use #runtime::threads::#registration_trait as _;
                    (&&#runtime::threads::#constructor(move || #setup_call, move |__airbug_input| #call))
                        .#registration_method(suite, &__airbug_name, #allocator, __airbug_workers, #policy);
                })
            } else if options.threads.is_some()
                && !is_async
                && options.setup.is_none()
                && options.inherited_thread_dispatch
            {
                quote!({
                    use #runtime::threads::RegisterInheritedAllocated as _;
                    (&&#runtime::threads::InheritedCallback::new(#thread_argument_marker, move || #invocation))
                        .register_inherited_allocated(suite, &__airbug_name, #allocator, __airbug_workers, #policy);
                })
            } else if options.threads.is_some()
                && is_async
                && options.setup.is_none()
                && options.inherited_thread_dispatch
            {
                quote!({
                    use #runtime::threads::RegisterInheritedAsyncAllocated as _;
                    (&&#runtime::threads::InheritedCallback::new(#thread_argument_marker, (move || #executor, async move || #invocation.await)))
                        .register_inherited_async_allocated(suite, &__airbug_name, #allocator, __airbug_workers, #policy);
                })
            } else if options.threads.is_some() && is_async {
                let method = format_ident!(
                    "bench_async_threads_allocated_with_{}input",
                    if owned_input { "owned_" } else { "" }
                );
                quote!(suite.#method(&__airbug_name, #allocator, __airbug_workers,
                    move || #executor, move || #setup_call,
                    async move |__airbug_input| #call.await, #policy);)
            } else if options.threads.is_some() {
                let method = format_ident!(
                    "bench_threads_allocated_with_{}{}input",
                    if worker_setup || options.setup.is_none() {
                        "local_"
                    } else {
                        ""
                    },
                    if owned_input { "owned_" } else { "" }
                );
                quote!(suite.#method(&__airbug_name, #allocator, __airbug_workers,
                    move || #setup_call, move |__airbug_input| #call, #policy);)
            } else if is_async {
                quote!(suite.#method(&__airbug_name, #allocator, move || #executor,
                    move || #setup_call, async move |__airbug_input| #call.await, #policy, #batch);)
            } else {
                quote!(suite.#method(&__airbug_name, #allocator,
                    move || #setup_call, move |__airbug_input| #call, #policy, #batch);)
            }
        } else if is_async {
            quote!(suite.bench_async_allocated(&__airbug_name, #allocator, move || #executor,
                async move || #invocation.await);)
        } else {
            quote!(suite.bench_allocated(&__airbug_name, #allocator, move || #invocation);)
        }
    } else if options.custom {
        let call = if has_args {
            quote!(#target(__airbug_iterations, ::std::hint::black_box(#arg)))
        } else {
            quote!(#target(__airbug_iterations))
        };
        if is_async {
            quote!(suite.bench_async_custom(&__airbug_name, move || #executor,
                async move |__airbug_iterations| #call.await);)
        } else {
            quote!(suite.bench_custom(&__airbug_name, move |__airbug_iterations| #call);)
        }
    } else if let Some(setup) = &options.setup {
        let setup_call = if has_args {
            if matches!(setup, Expr::Closure(closure) if closure.inputs.iter().any(|input| !matches!(input, syn::Pat::Type(_))))
            {
                quote!({
                    fn __airbug_setup<A, I>(setup: impl FnOnce(A) -> I, value: A) -> I { setup(value) }
                    __airbug_setup(#setup, __airbug_arg)
                })
            } else {
                quote!({
                    fn __airbug_setup<A: Copy, V: ::std::borrow::Borrow<A>, I>(
                        setup: impl FnOnce(A) -> I, value: &V,
                    ) -> I { setup(*value.borrow()) }
                    __airbug_setup(#setup, &__airbug_arg)
                })
            }
        } else {
            quote!((#setup)())
        };
        let setup_call = if input_counts.is_empty() {
            setup_call
        } else {
            let record = input_counts.iter().map(|(unit, _)| {
                let name = format_ident!("__airbug_input_count_{unit}");
                let name = if is_async && options.inherited_thread_dispatch {
                    quote!((#name.lock().expect("input counter mutex poisoned")))
                } else {
                    quote!(#name)
                };
                quote!(
                    __airbug_setup_counters.add(#unit, #name(&__airbug_input));
                )
            });
            quote!({ let __airbug_input = #setup_call; #(#record)* __airbug_input })
        };
        if !is_async && options.inherited_thread_dispatch && !worker_setup {
            let (constructor, trait_name, method) = if owned_input {
                (
                    format_ident!("inherited_owned_setup"),
                    format_ident!("RegisterInheritedOwnedSetup"),
                    format_ident!("register_inherited_owned_setup"),
                )
            } else {
                (
                    format_ident!("inherited_setup"),
                    format_ident!("RegisterInheritedSetup"),
                    format_ident!("register_inherited_setup"),
                )
            };
            quote!({
                use #runtime::threads::#trait_name as _;
                (&&#runtime::threads::#constructor(move || #setup_call, |__airbug_input| #target(__airbug_input)))
                    .#method(suite, &__airbug_name, __airbug_workers, #policy);
            })
        } else if is_async && options.inherited_thread_dispatch {
            let (constructor, trait_name, method) = if owned_input {
                (
                    format_ident!("inherited_async_owned_setup"),
                    format_ident!("RegisterInheritedAsyncOwnedSetup"),
                    format_ident!("register_inherited_async_owned_setup"),
                )
            } else {
                (
                    format_ident!("inherited_async_setup"),
                    format_ident!("RegisterInheritedAsyncSetup"),
                    format_ident!("register_inherited_async_setup"),
                )
            };
            let receiver = if input_counts
                .iter()
                .any(|(_, expr)| !matches!(expr, Expr::Closure(_) | Expr::Path(_)))
            {
                quote!(&__airbug_callback)
            } else {
                quote!(&&__airbug_callback)
            };
            quote!({
                use #runtime::threads::#trait_name as _;
                let __airbug_callback = #runtime::threads::#constructor(#thread_argument_marker, move || #executor,
                    move || #setup_call, async move |__airbug_input| #target(__airbug_input).await);
                (#receiver).#method(suite, &__airbug_name, __airbug_workers, #policy);
            })
        } else if is_async && options.threads.is_some() {
            if owned_input {
                quote!(suite.bench_async_threads_with_owned_input(&__airbug_name, __airbug_workers,
                    move || #executor, move || #setup_call,
                    async move |__airbug_input| #target(__airbug_input).await, #policy);)
            } else {
                quote!(suite.bench_async_threads_with_input(&__airbug_name, __airbug_workers,
                    move || #executor, move || #setup_call,
                    async move |__airbug_input| #target(__airbug_input).await, #policy);)
            }
        } else if owned_input && is_async {
            if let Some(batch) = &options.batch {
                quote!(suite.bench_async_batched(&__airbug_name, move || #executor, move || #setup_call,
                    async move |__airbug_input| #target(__airbug_input).await, #policy, #batch);)
            } else {
                quote!(suite.bench_async_with_owned_input(&__airbug_name, move || #executor, move || #setup_call,
                move |__airbug_input| #target(__airbug_input), #policy);)
            }
        } else if owned_input && worker_setup && options.threads.is_some() {
            quote!(suite.bench_threads_with_local_owned_input(&__airbug_name, __airbug_workers, move || #setup_call,
                |__airbug_input| #target(__airbug_input), #policy);)
        } else if owned_input && options.threads.is_some() {
            quote!(suite.bench_threads_with_owned_input(&__airbug_name, __airbug_workers, move || #setup_call,
                |__airbug_input| #target(__airbug_input), #policy);)
        } else if owned_input {
            if let Some(batch) = &options.batch {
                quote!(suite.bench_batched(&__airbug_name, move || #setup_call,
                    |__airbug_input| #target(__airbug_input), #policy, #batch);)
            } else {
                quote!(suite.bench_with_owned_input(&__airbug_name, move || #setup_call,
                    |__airbug_input| #target(__airbug_input), #policy);)
            }
        } else if is_async {
            if let Some(batch) = &options.batch {
                quote!(suite.bench_async_batched_ref(&__airbug_name, move || #executor, move || #setup_call,
                    async move |__airbug_input| #target(__airbug_input).await, #policy, #batch);)
            } else {
                quote!(suite.bench_async_with_input(&__airbug_name, move || #executor, move || #setup_call,
                async move |__airbug_input| #target(__airbug_input).await, #policy);)
            }
        } else if worker_setup && options.threads.is_some() {
            quote!(suite.bench_threads_with_local_input(&__airbug_name, __airbug_workers, move || #setup_call,
                |__airbug_input| #target(__airbug_input), #policy);)
        } else if options.threads.is_some() {
            quote!(suite.bench_threads_with_input(&__airbug_name, __airbug_workers, move || #setup_call,
                |__airbug_input| #target(__airbug_input), #policy);)
        } else if let Some(batch) = &options.batch {
            quote!(suite.bench_batched_ref(&__airbug_name, move || #setup_call,
                |__airbug_input| #target(__airbug_input), #policy, #batch);)
        } else {
            quote!(suite.bench_with_input(&__airbug_name, move || #setup_call,
                |__airbug_input| #target(__airbug_input), #policy);)
        }
    } else if is_async
        && borrowed_arg
        && let Some(batch) = &options.batch
    {
        quote!(suite.bench_async_with_value_batched(&__airbug_name, move || #executor, __airbug_arg,
            async move |__airbug_input| #target(__airbug_input).await, #policy, #batch);)
    } else if let Some(batch) = &options.batch {
        if is_async {
            quote!(suite.bench_async_batched(&__airbug_name, move || #executor, || (),
                async move |_: ()| #invocation.await, #policy, #batch);)
        } else {
            quote!(suite.bench_batched(&__airbug_name, || (),
                move |_: ()| #invocation, #policy, #batch);)
        }
    } else if is_async && options.threads.is_some() && options.inherited_thread_dispatch {
        quote!({
            use #runtime::threads::RegisterInheritedAsync as _;
            (&&#runtime::threads::InheritedCallback::new(#thread_argument_marker, (move || #executor, async move || #invocation.await)))
                .register_inherited_async(suite, &__airbug_name, __airbug_workers, #policy);
        })
    } else if is_async && options.threads.is_some() {
        quote!(suite.bench_async_threads(&__airbug_name, __airbug_workers, move || #executor,
            async move || #invocation.await, #policy);)
    } else if is_async && borrowed_arg {
        quote!(suite.bench_async_with_value(&__airbug_name, move || #executor, __airbug_arg,
            async move |__airbug_input| #target(__airbug_input).await, #policy);)
    } else if is_async {
        quote!(suite.bench_async_factory(&__airbug_name, move || #executor, move || #invocation, #policy);)
    } else if options.threads.is_some() && options.inherited_thread_dispatch {
        quote!({
            use #runtime::threads::RegisterInheritedCallback as _;
            (&&#runtime::threads::InheritedCallback::new(#thread_argument_marker, move || #invocation))
                .register_inherited(suite, &__airbug_name, __airbug_workers, #policy);
        })
    } else if options.threads.is_some() {
        if outside || options.drop_output.is_none() {
            quote!(suite.bench_threads_with_local_input(&__airbug_name, __airbug_workers, || (), move |_: &mut ()| #invocation, #policy);)
        } else {
            quote!({
                if suite.has_registration_batch_policy() {
                    suite.bench_threads_with_local_input(&__airbug_name, __airbug_workers, || (), move |_: &mut ()| #invocation, #policy);
                } else {
                    suite.bench_threads(&__airbug_name, __airbug_workers, move || #invocation);
                }
            })
        }
    } else if outside || options.drop_output.is_none() {
        quote!(suite.bench_with_input(&__airbug_name, || (), move |_: &mut ()| #invocation, #policy);)
    } else {
        quote!(suite.bench(&__airbug_name, move || #invocation);)
    };
    // Keep the original plain loop when no deferred destruction is requested.
    let body = if !options.inherited_thread_dispatch
        && options.drop_output.is_none()
        && options.setup.is_none()
        && options.batch.is_none()
        && options.measurement.is_none()
        && !options.custom
    {
        let plain = if let Some(allocator) = &options.allocator {
            if options.threads.is_some() {
                None
            } else if is_async {
                Some(
                    quote!(suite.bench_async_allocated(&__airbug_name, #allocator, move || #executor, async move || #invocation.await);),
                )
            } else {
                Some(
                    quote!(suite.bench_allocated(&__airbug_name, #allocator, move || #invocation);),
                )
            }
        } else if is_async {
            None
        } else if options.threads.is_some() {
            Some(
                quote!(suite.bench_threads(&__airbug_name, __airbug_workers, move || #invocation);),
            )
        } else {
            Some(quote!(suite.bench(&__airbug_name, move || #invocation);))
        };
        if let Some(plain) = plain {
            quote! { if matches!(#policy, #runtime::DropPolicy::InsideTiming) && !suite.has_registration_batch_policy() { #plain } else { #body } }
        } else {
            body
        }
    } else {
        body
    };
    let mut counts = Vec::new();
    let mut counters = Vec::new();
    for (unit, expression) in [
        ("bytes", &options.bytes),
        ("items", &options.items),
        ("chars", &options.chars),
        ("cycles", &options.cycles),
        ("bits", &options.bits),
    ] {
        if let Some(count) = expression {
            let variable = format_ident!("__airbug_count_{}", unit);
            let count = if has_args && matches!(count, Expr::Closure(_)) {
                quote!((#count)(#arg))
            } else {
                quote!(#count)
            };
            counts.push(quote!(let #variable: u64 = #count;));
            counters.push(quote!(suite.work_units(#unit, #variable);));
        }
    }
    let ignored = options
        .ignore
        .as_ref()
        .map(|value| quote!(suite.ignore(#value);));
    let comparison = comparison_defaults(&options, &runtime);
    let bootstrap = bootstrap_defaults(&options, &runtime);
    let sampling = sampling_defaults(&options, &runtime);
    let overhead = options.overhead.as_ref().map(|value| {
        let enabled = value.value() == "subtract";
        quote!(suite.compensate_case(#enabled);)
    });
    let summary_scale = summary_scale_default(&options, &runtime)
        .map(|scale| quote!(suite.summary_scale_case(#scale);));
    let timer = timer_defaults(&options, &runtime).map(|t| quote!(suite.timer_case(#t);));
    let quick = quick_defaults(&options, &runtime).map(|q| quote!(suite.quick_case(#q);));
    let summary_family = options
        .summary_family
        .as_ref()
        .map(|name| quote!(suite.summary_family(#name);));
    let argument_contract = has_args.then(|| quote!(suite.parameter("arg", &__airbug_label);));
    let location = quote::quote_spanned! {f.sig.ident.span()=>
        suite.source_location(file!(), line!(), column!());
    };
    let family = (!generic_names.is_empty()).then(|| quote!(suite.ordering_family(#name);));
    let validate_drop = options.custom.then(|| quote! {
        assert!(!suite.has_registration_drop_policy(), "custom timing owns output destruction; imported drop_output defaults cannot apply");
    });
    let mut per_case = quote! {
        #validate_drop
        #(#counts)*
        #input_counter_init
        #body
        #input_counter_attach
        #(#counters)*
        #ignored
        #argument_contract
        #location
        #family
        suite.sampling(#sampling);
        suite.comparison_case(#comparison);
        suite.bootstrap_case(#bootstrap);
        #quick
        #timer
        #summary_scale
        #summary_family
        #overhead
    };
    if let Some(threads) = &options.threads {
        let threads = if matches!(threads, Expr::Lit(expr) if matches!(expr.lit, syn::Lit::Int(_)))
        {
            quote!({ let workers: usize = #threads; workers })
        } else {
            quote!(#threads)
        };
        let share = borrowed_arg
            .then(|| quote!(let __airbug_shared = ::std::sync::Arc::new(__airbug_arg);));
        let clone = borrowed_arg
            .then(|| quote!(let __airbug_arg = ::std::sync::Arc::clone(&__airbug_shared);));
        per_case = quote! {
            #share
            for __airbug_workers in suite.resolve_thread_counts(#threads) {
                #clone
                let __airbug_name = ::std::format!("{}/threads={}", __airbug_name, __airbug_workers);
                #per_case
            }
        };
    }
    // Generic dimensions are recorded on every registered case, after runtime dimensions.
    // Their values are also part of the ID, so no specialization can silently overwrite another.
    let mut generic_label = quote!(let __airbug_base = ::std::string::String::from(#name););
    for param in &type_params {
        let id = &param.ident;
        generic_label.extend(quote!(let __airbug_base = ::std::format!("{}/type={}", __airbug_base, ::std::any::type_name::<#id>());));
    }
    for param in &const_params {
        let id = &param.ident;
        generic_label.extend(
            quote!(let __airbug_base = ::std::format!("{}/const={:?}", __airbug_base, #id);),
        );
    }
    let core = if let Some(args) = &options.args {
        let copy = if borrowed_arg {
            quote!()
        } else if options.setup.is_none()
            && let Some(syn::FnArg::Typed(argument)) = f.sig.inputs.last()
            && !matches!(&*argument.ty, Type::ImplTrait(_))
        {
            let ty = &argument.ty;
            quote!(
                fn __airbug_copy<T: Copy>(value: &T) -> T { *value }
                let __airbug_arg: #ty = __airbug_copy(::std::borrow::Borrow::<#ty>::borrow(&__airbug_arg));
            )
        } else {
            quote!(fn __airbug_copy<T: Copy>(value: T) -> T { value } let __airbug_arg = __airbug_copy(__airbug_arg);)
        };
        // Record args before body moves them into the workload closure.
        quote! {
            #generic_label
            // Autoref selection keeps existing Debug labels, with ToString (including Display) as fallback.
            struct __AirbugArgumentLabel<'a, T: ?Sized>(&'a T);
            trait __AirbugLabel {
                fn __airbug_label(self) -> ::std::string::String;
            }
            impl<T: ::std::fmt::Debug + ?Sized> __AirbugLabel for &&__AirbugArgumentLabel<'_, T> {
                fn __airbug_label(self) -> ::std::string::String { ::std::format!("{:?}", self.0) }
            }
            impl<T: ::std::string::ToString + ?Sized> __AirbugLabel for &__AirbugArgumentLabel<'_, T> {
                fn __airbug_label(self) -> ::std::string::String { ::std::string::ToString::to_string(self.0) }
            }
            for __airbug_arg in #args {
                #copy
                let __airbug_label = (&&__AirbugArgumentLabel(&__airbug_arg)).__airbug_label();
                let __airbug_name = ::std::format!("{}/{}", __airbug_base, __airbug_label);
                #per_case
            }
        }
    } else {
        quote! { #generic_label let __airbug_name = __airbug_base; #per_case }
    };
    if generic_names.is_empty() {
        return syn::parse2(quote!({ #core }));
    }

    let mut helper_generics = f.sig.generics.clone();
    let bounds: Vec<_> = helper_generics
        .type_params_mut()
        .map(|param| {
            let mut bounds = std::mem::take(&mut param.bounds);
            bounds.push(syn::parse_quote!('static));
            (param.ident.clone(), bounds)
        })
        .collect();
    for (ident, bounds) in bounds {
        let predicates = &mut helper_generics.make_where_clause().predicates;
        let existing = predicates.iter_mut().find_map(|predicate| match predicate {
            syn::WherePredicate::Type(p)
                if matches!(&p.bounded_ty,
                Type::Path(path) if path.qself.is_none() && path.path.is_ident(&ident)) =>
            {
                Some(p)
            }
            _ => None,
        });
        if let Some(existing) = existing {
            existing.bounds.extend(bounds);
        } else {
            predicates.push(syn::parse_quote!(#ident: #bounds));
        }
    }
    let (impl_generics, _, where_clause) = helper_generics.split_for_impl();
    let types: Vec<Vec<_>> = if let Some(types) = &options.types {
        types
            .iter()
            .map(|ty| {
                if type_params.len() == 1 {
                    Ok(vec![quote!(#ty)])
                } else if let Type::Tuple(tuple) = ty {
                    if tuple.elems.len() != type_params.len() {
                        return Err(syn::Error::new_spanned(
                            ty,
                            "type tuple must have one type per generic type parameter",
                        ));
                    }
                    Ok(tuple.elems.iter().map(|t| quote!(#t)).collect())
                } else {
                    Err(syn::Error::new_spanned(
                        ty,
                        "multiple type parameters require tuple rows: types = [(T1, T2), ...]",
                    ))
                }
            })
            .collect::<syn::Result<_>>()?
    } else {
        vec![vec![]]
    };
    let external_consts = options
        .consts
        .as_ref()
        .filter(|e| !matches!(e, Expr::Array(_)));
    let consts: Vec<Vec<_>> = if let Some(list) = external_consts {
        // Rust requires each specialization to be a compile-time expression.
        // Unroll a bounded list, using a valid fallback for unused slots; guards
        // below ensure those slots never register or evaluate runtime arguments.
        (0usize..20)
            .map(|index| {
                let value = quote!((#list)[if #index < (#list).len() { #index } else { 0 }]);
                if const_params.len() == 1 {
                    vec![quote!({ #value })]
                } else {
                    (0..const_params.len())
                        .map(|field| {
                            let field = syn::Index::from(field);
                            quote!({ (#value).#field })
                        })
                        .collect()
                }
            })
            .collect()
    } else if let Some(Expr::Array(consts)) = &options.consts {
        consts
            .elems
            .iter()
            .map(|value| {
                if const_params.len() == 1 {
                    Ok(vec![quote!({ #value })])
                } else if let Expr::Tuple(tuple) = value {
                    if tuple.elems.len() != const_params.len() {
                        return Err(syn::Error::new_spanned(
                            value,
                            "const tuple must have one value per generic const parameter",
                        ));
                    }
                    Ok(tuple.elems.iter().map(|e| quote!({ #e })).collect())
                } else {
                    Err(syn::Error::new_spanned(
                        value,
                        "multiple const parameters require tuple rows: consts = [(N, M), ...]",
                    ))
                }
            })
            .collect::<syn::Result<_>>()?
    } else {
        vec![vec![]]
    };
    let concrete_lifetimes = f.sig.generics.lifetimes().next().is_some();
    let mut calls = Vec::new();
    for ty in &types {
        for (index, constant) in consts.iter().enumerate() {
            let mut type_arguments = ty.iter();
            let mut const_arguments = constant.iter();
            let arguments: Vec<_> = f
                .sig
                .generics
                .params
                .iter()
                .filter_map(|p| match p {
                    syn::GenericParam::Type(_) => Some(type_arguments.next().unwrap().clone()),
                    syn::GenericParam::Const(_) => Some(const_arguments.next().unwrap().clone()),
                    syn::GenericParam::Lifetime(_) => None,
                })
                .collect();
            let call = if concrete_lifetimes {
                // Let Rust check the original function's bounds with concrete types.
                // Repeating a lifetime bound on a generic registrar would incorrectly
                // require a short borrow of captured input to outlive that registrar.
                let mut definitions = Vec::new();
                let mut aliases = Vec::new();
                for (index, (param, ty)) in type_params.iter().zip(ty).enumerate() {
                    let mut ty: Type = syn::parse2(ty.clone())?;
                    syn::visit_mut::VisitMut::visit_type_mut(&mut AliasLifetimes, &mut ty);
                    let temporary = format_ident!("__AirbugType{index}");
                    let ident = &param.ident;
                    definitions.push(quote!(type #temporary = #ty;));
                    aliases.push(quote!(type #ident = #temporary;));
                }
                for (index, (param, value)) in const_params.iter().zip(constant).enumerate() {
                    let temporary = format_ident!("__AIRBUG_CONST_{index}");
                    let ident = &param.ident;
                    let ty = &param.ty;
                    definitions.push(quote!(const #temporary: #ty = #value;));
                    aliases.push(quote!(const #ident: #ty = #temporary;));
                }
                // Separate lexical scopes avoid recursive aliases when an external
                // type/constant has the same name as the benchmark parameter.
                quote!({ #(#definitions)* { #(#aliases)* #core } })
            } else {
                quote!(__airbug_specialize::<#(#arguments),*>(suite);)
            };
            calls.push(if let Some(list) = external_consts {
                quote!(if #index < (#list).len() { #call })
            } else {
                call
            });
        }
    }
    let validate_consts = external_consts.map(|list| {
        let fields: Vec<_> = const_params.iter().map(|p| &p.ty).collect();
        let element = if fields.len() == 1 {
            let ty = fields[0];
            quote!(#ty)
        } else {
            quote!((#(#fields),*))
        };
        quote!(const {
        let _: &[#element] = &(#list);
        assert!((#list).len() >= 1 && (#list).len() <= 20,
            "external const lists require 1..=20 entries");
    };)
    });
    let helper = (!concrete_lifetimes).then(|| quote!(
        fn __airbug_specialize #impl_generics (suite: &mut #runtime::Suite<'_>) #where_clause { #core }
    ));
    syn::parse2(quote!({
        #validate_consts
        #helper
        #(#calls)*
    }))
}

/// Register a benchmark through `register_NAME(&mut suite)`.
#[proc_macro_attribute]
pub fn bench(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut f = syn::parse_macro_input!(item as ItemFn);
    let options = syn::parse::<Options>(args).and_then(|mut options| {
        take_ignore(&mut f.attrs, &mut options)?;
        Ok(options)
    });
    let options = match options {
        Ok(options) => options,
        Err(error) => return error.to_compile_error().into(),
    };
    let runtime = match runtime_path(&options) {
        Ok(path) => path,
        Err(error) => return error.to_compile_error().into(),
    };
    let block = match registration(&f, options) {
        Ok(block) => block,
        Err(error) => return error.to_compile_error().into(),
    };
    let register = format_ident!("register_{}", f.sig.ident);
    let visibility = &f.vis;
    let cfg = f.attrs.iter().filter(|attr| attr.path().is_ident("cfg"));
    quote! { #f #(#cfg)* #visibility fn #register(suite: &mut #runtime::Suite<'_>) #block }.into()
}

impl Options {
    fn inherit(&mut self, parent: &Self) {
        if self.sample_ms.is_none() && self.measurement_ms.is_none() {
            self.sample_ms = parent.sample_ms.clone();
            self.measurement_ms = parent.measurement_ms.clone();
        }
        macro_rules! inherit { ($($field:ident),*) => { $(if self.$field.is_none() { self.$field = parent.$field.clone(); })* }; }
        inherit!(
            crate_path,
            allocator,
            measurement,
            formatter,
            timer,
            summary_scale,
            summary_family,
            overhead,
            significance_level,
            noise_threshold_percent,
            hypothesis_resamples,
            hypothesis_seed,
            resamples,
            confidence_level,
            analysis_seed,
            quick,
            quick_config,
            executor,
            threads,
            setup_thread,
            bytes,
            items,
            chars,
            cycles,
            bits,
            input_bytes,
            input_items,
            input_chars,
            input_cycles,
            input_bits,
            samples,
            sample_count_unit,
            worker_start,
            warmup_ms,
            iterations,
            sampling,
            min_time_ms,
            max_time_ms,
            exclude_external_time,
            drop_output,
            batch,
            ignore
        );
    }
    fn validate_imported_defaults(&self, location: &syn::Ident) -> syn::Result<()> {
        if self.groups.as_ref().is_none_or(|groups| groups.is_empty()) {
            return Ok(());
        }
        let unsupported: Vec<_> = [
            ("executor", self.executor.is_some()),
            ("allocator", self.allocator.is_some()),
            ("setup_thread", self.setup_thread.is_some()),
            ("input_bytes", self.input_bytes.is_some()),
            ("input_items", self.input_items.is_some()),
            ("input_chars", self.input_chars.is_some()),
            ("input_cycles", self.input_cycles.is_some()),
            ("input_bits", self.input_bits.is_some()),
        ]
        .into_iter()
        .filter_map(|(name, present)| present.then_some(name))
        .collect();
        if unsupported.is_empty() {
            return Ok(());
        }
        Err(syn::Error::new_spanned(
            location,
            format!(
                "defaults cannot yet cross imported groups: {}; declare these options on the imported group",
                unsupported.join(", ")
            ),
        ))
    }

    fn validate_group(&self) -> syn::Result<()> {
        if self.args.is_some()
            || self.setup.is_some()
            || self.types.is_some()
            || self.consts.is_some()
            || self.custom
        {
            return Err(syn::Error::new(
                proc_macro::Span::call_site().into(),
                "groups accept name, sampling, threads, counters, allocator, executor, batch, drop_output and ignore defaults",
            ));
        }
        Ok(())
    }
}

fn take_ignore(attrs: &mut Vec<syn::Attribute>, options: &mut Options) -> syn::Result<()> {
    if let Some(index) = attrs.iter().position(|attr| attr.path().is_ident("ignore")) {
        let attr = attrs.remove(index);
        if options.ignore.is_some() {
            return Err(syn::Error::new_spanned(attr, "ignore specified twice"));
        }
        options.ignore = Some(syn::parse_quote!(true));
    }
    Ok(())
}

fn marker(attr: &syn::Attribute, expected: &str, runtime: &syn::Path) -> bool {
    let path = attr.path();
    let matches = |runtime: &syn::Path| {
        path.segments.len() == runtime.segments.len() + 1
            && path
                .segments
                .last()
                .is_some_and(|segment| segment.ident == expected)
            && path
                .segments
                .iter()
                .zip(&runtime.segments)
                .all(|(a, b)| a.ident == b.ident)
    };
    path.is_ident(expected)
        || matches(runtime)
        || runtime_path(&Options::default()).is_ok_and(|path| matches(&path))
}
fn attribute_options(attribute: syn::Attribute) -> syn::Result<Options> {
    match attribute.meta {
        Meta::Path(_) => Ok(Options::default()),
        Meta::List(list) => list.parse_args::<Options>(),
        meta => Err(syn::Error::new_spanned(
            meta,
            "expected an attribute or attribute(options)",
        )),
    }
}

fn register_module(module: &mut syn::ItemMod, inherited: &Options) -> syn::Result<()> {
    let runtime = runtime_path(inherited)?;
    let Some((_, items)) = &mut module.content else {
        return Err(syn::Error::new_spanned(
            module,
            "suite/group requires an inline module",
        ));
    };
    let mut registrations = Vec::new();
    for item in items.iter_mut() {
        match item {
            syn::Item::Fn(function) => {
                let Some(index) = function
                    .attrs
                    .iter()
                    .position(|attr| marker(attr, "bench", &runtime))
                else {
                    continue;
                };
                let mut options = attribute_options(function.attrs.remove(index))?;
                take_ignore(&mut function.attrs, &mut options)?;
                let explicit_executor = options.executor.is_some();
                options.inherit(inherited);
                // Executor defaults apply only to async members of mixed groups.
                // An explicitly written executor on a sync function remains an error.
                if function.sig.asyncness.is_none() && !explicit_executor {
                    options.executor = None;
                }
                let block = registration(function, options)?;
                let cfg = function
                    .attrs
                    .iter()
                    .filter(|attr| attr.path().is_ident("cfg"));
                registrations.push(quote!(#(#cfg)* #block));
            }
            syn::Item::Mod(child) if child.content.is_some() => {
                let mut options = if let Some(index) = child
                    .attrs
                    .iter()
                    .position(|attr| marker(attr, "group", &runtime))
                {
                    attribute_options(child.attrs.remove(index))?
                } else {
                    Options::default()
                };
                take_ignore(&mut child.attrs, &mut options)?;
                options.validate_group()?;
                options.inherit(inherited);
                let name = options.name.clone().unwrap_or_else(|| {
                    syn::LitStr::new(&child.ident.to_string(), child.ident.span())
                });
                register_module(child, &options)?;
                let ident = &child.ident;
                let cfg = child
                    .attrs
                    .iter()
                    .filter(|attr| attr.path().is_ident("cfg"));
                registrations.push(
                    quote! { #(#cfg)* { suite.group(#name, #ident::__airbug_register_group); } },
                );
            }
            _ => {}
        }
    }
    inherited.validate_imported_defaults(&module.ident)?;
    if let Some(groups) = &inherited.groups {
        if inherited.measurement.is_some() || inherited.formatter.is_some() {
            return Err(syn::Error::new_spanned(
                &module.ident,
                "measurement/formatter defaults must be declared on the imported group",
            ));
        }
        let sampling = sampling_defaults(inherited, &runtime);
        let ignored = inherited
            .ignore
            .as_ref()
            .map(|v| quote!(Some(#v)))
            .unwrap_or_else(|| quote!(None));
        let mut counters = Vec::new();
        for (unit, value) in [
            ("bytes", &inherited.bytes),
            ("items", &inherited.items),
            ("chars", &inherited.chars),
            ("cycles", &inherited.cycles),
            ("bits", &inherited.bits),
        ] {
            if let Some(value) = value {
                if matches!(value, Expr::Closure(_)) {
                    return Err(syn::Error::new_spanned(
                        value,
                        "argument-dependent counter defaults cannot yet cross imported groups; declare them on the imported group",
                    ));
                }
                counters.push(quote!((#unit, #value)));
            }
        }
        for path in groups {
            let registration = quote! { suite.group_with_defaults(#path::__AIRBUG_GROUP_NAME, #sampling, #ignored, &[#(#counters),*], #path::__airbug_register_group); };
            let registration = if let Some(threads) = &inherited.threads {
                let counts = if matches!(threads, Expr::Lit(expr) if matches!(&expr.lit, syn::Lit::Bool(value) if !value.value))
                {
                    quote!(None)
                } else if matches!(threads, Expr::Lit(expr) if matches!(expr.lit, syn::Lit::Int(_)))
                {
                    quote!(Some(#runtime::threads::counts({ let workers: usize = #threads; workers })))
                } else {
                    quote!(Some(#runtime::threads::counts(#threads)))
                };
                quote! { suite.with_thread_defaults(#counts, |suite| { #registration }); }
            } else {
                registration
            };
            let registration = if let Some(batch) = &inherited.batch {
                quote! { suite.with_batch_defaults(#batch, |suite| { #registration }); }
            } else {
                registration
            };
            let registration = if let Some(drop) = &inherited.drop_output {
                let policy = if drop.value() == "outside" {
                    quote!(#runtime::DropPolicy::OutsideTiming)
                } else {
                    quote!(#runtime::DropPolicy::InsideTiming)
                };
                quote! { suite.with_drop_defaults(#policy, |suite| { #registration }); }
            } else {
                registration
            };
            let registration = if let Some(name) = &inherited.summary_family {
                quote! { suite.with_summary_family_defaults(#name, |suite| { #registration }); }
            } else {
                registration
            };
            let registration = if let Some(scale) = summary_scale_default(inherited, &runtime) {
                quote! { suite.with_summary_scale_defaults(#scale, |suite| { #registration }); }
            } else {
                registration
            };
            let registration = if let Some(timer) = timer_defaults(inherited, &runtime) {
                quote! { suite.with_timer_defaults(#timer, |suite| { #registration }); }
            } else {
                registration
            };
            let registration = if let Some(value) = &inherited.overhead {
                let enabled = value.value() == "subtract";
                quote! { suite.with_overhead_defaults(#enabled, |suite| { #registration }); }
            } else {
                registration
            };
            let bootstrap = bootstrap_defaults(inherited, &runtime);
            let registration =
                quote! { suite.with_bootstrap_defaults(#bootstrap, |suite| { #registration }); };
            let comparison = comparison_defaults(inherited, &runtime);
            let registration =
                quote! { suite.with_comparison_defaults(#comparison, |suite| { #registration }); };
            registrations.push(if let Some(quick) = quick_defaults(inherited, &runtime) {
                quote! { suite.with_quick_defaults(#quick, |suite| { #registration }); }
            } else {
                registration
            });
        }
    }
    let group_name = inherited
        .name
        .clone()
        .unwrap_or_else(|| syn::LitStr::new(&module.ident.to_string(), module.ident.span()));
    items.push(syn::parse_quote! {
        #[doc(hidden)]
        pub const __AIRBUG_GROUP_NAME: &str = #group_name;
    });
    let group_location = quote::quote_spanned! { module.ident.span() =>
        suite.group_source_location(file!(), line!(), column!());
    };
    items.push(syn::parse_quote! {
        #[doc(hidden)]
        pub fn __airbug_register_group(suite: &mut #runtime::Suite<'_>) {
            #group_location
            #(#registrations)*
        }
    });
    Ok(())
}

/// Generate a Cargo benchmark executable, recursively registering inline groups.
#[proc_macro_attribute]
pub fn suite(args: TokenStream, item: TokenStream) -> TokenStream {
    let options = syn::parse_macro_input!(args as Options);
    let runtime = match runtime_path(&options) {
        Ok(path) => path,
        Err(error) => return error.to_compile_error().into(),
    };
    let mut module = syn::parse_macro_input!(item as syn::ItemMod);
    if let Err(error) = options
        .validate_group()
        .and_then(|_| register_module(&mut module, &options))
    {
        return error.to_compile_error().into();
    }
    let ident = &module.ident;
    let name = options
        .name
        .unwrap_or_else(|| syn::LitStr::new(&ident.to_string(), ident.span()));
    quote! {
        #module
        fn main() -> ::std::process::ExitCode {
            let mut suite = #runtime::Suite::new(#name);
            if cfg!(test) {
                suite.cargo_harness();
            }
            match suite.main_registered(#ident::__airbug_register_group) {
                Ok(()) => ::std::process::ExitCode::SUCCESS,
                Err(error) => {
                    ::std::eprintln!("error: {error}");
                    ::std::process::ExitCode::FAILURE
                }
            }
        }
    }
    .into()
}

/// A reusable benchmark group. It may live in another source file or library crate.
/// Import it with `#[suite(groups = [path::to::group])]`.
#[proc_macro_attribute]
pub fn group(args: TokenStream, item: TokenStream) -> TokenStream {
    let options = syn::parse_macro_input!(args as Options);
    let mut module = syn::parse_macro_input!(item as syn::ItemMod);
    if let Err(error) = options
        .validate_group()
        .and_then(|_| register_module(&mut module, &options))
    {
        return error.to_compile_error().into();
    }
    quote!(#module).into()
}

#[cfg(test)]
mod summary_scale_tests {
    use super::Options;
    #[test]
    fn validates_summary_scale_values_and_duplicates() {
        for value in ["linear", "logarithmic"] {
            assert!(syn::parse_str::<Options>(&format!("summary_scale = \"{value}\"")).is_ok());
        }
        for source in [
            "summary_scale = \"log\"",
            "summary_scale = 1",
            "summary_scale = \"linear\", summary_scale = \"logarithmic\"",
        ] {
            assert!(syn::parse_str::<Options>(source).is_err(), "{source}");
        }
    }
}

#[cfg(test)]
mod imported_default_tests {
    use super::Options;
    #[test]
    fn unsupported_imported_defaults_are_never_silently_discarded() {
        for option in [
            "executor = executor()",
            "allocator = &ALLOCATOR",
            "setup_thread = \"worker\"",
            "input_bytes = |v| v.len() as u64",
            "input_items = |v| 1",
            "input_chars = |v| 1",
            "input_cycles = |v| 1",
            "input_bits = |v| 1",
        ] {
            let source = format!("groups = [external], {option}");
            let options = syn::parse_str::<Options>(&source).unwrap();
            let error = options
                .validate_imported_defaults(&syn::parse_quote!(group))
                .unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains(option.split(" =").next().unwrap())
            );
            let local = syn::parse_str::<Options>(option).unwrap();
            assert!(
                local
                    .validate_imported_defaults(&syn::parse_quote!(group))
                    .is_ok()
            );
        }
        let supported = syn::parse_str::<Options>("groups = [external], samples = 3, bytes = 0, ignore = false, summary_scale = \"linear\"").unwrap();
        assert!(
            supported
                .validate_imported_defaults(&syn::parse_quote!(group))
                .is_ok()
        );
    }
}
