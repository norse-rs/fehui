#![recursion_limit = "128"]

extern crate proc_macro;
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

enum BuildPath {
    Str(String),
    Default,
}

impl BuildPath {
    fn as_str(&self) -> &str {
        match self {
            BuildPath::Str(s) => s.as_str(),
            BuildPath::Default => "build",
        }
    }
}

impl darling::FromMeta for BuildPath {
    fn from_word() -> darling::Result<Self> {
        Ok(BuildPath::Default)
    }
    fn from_string(s: &str) -> darling::Result<Self> {
        Ok(BuildPath::Str(s.to_string()))
    }
}

enum StatePath {
    Str(String),
    Default,
}

impl StatePath {
    fn as_str(&self) -> &str {
        match self {
            StatePath::Str(s) => s.as_str(),
            StatePath::Default => "state",
        }
    }
}

impl<'a> darling::FromMeta for StatePath {
    fn from_word() -> darling::Result<Self> {
        Ok(StatePath::Default)
    }
    fn from_string(s: &str) -> darling::Result<Self> {
        Ok(StatePath::Str(s.to_string()))
    }
}

#[derive(FromDeriveInput)]
#[darling(attributes(widget))]
struct WidgetOptions {
    build: BuildPath,
    #[darling(default)]
    state: Option<StatePath>,
}

#[proc_macro_derive(Widget, attributes(widget))]
pub fn widget(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let parser: WidgetOptions = FromDeriveInput::from_derive_input(&input).unwrap();

    let ident = &input.ident;

    let build_fn = proc_macro2::Ident::new(parser.build.as_str(), proc_macro2::Span::call_site());

    TokenStream::from(match parser.state {
        Some(state) => {
            let state_fn = proc_macro2::Ident::new(state.as_str(), proc_macro2::Span::call_site());
            quote!(impl Widget for #ident {
                fn build(&self, mut ctxt: Build) {
                    let state_ref = ctxt.create_state(|| Self::#state_fn(self));
                    let state_borrow = state_ref.borrow();
                    let state_inner = state_borrow.downcast_ref().unwrap();
                    let repr = Self::#build_fn(self, state_inner, &mut ctxt);
                    unsafe { ctxt.add(repr, None); }
                }
                fn layout(&self, _: Layouter) -> Layout {
                    Layout::Pass
                }
            })
        }
        None => quote!(impl Widget for #ident {
            fn build(&self, mut ctxt: Build) {
                let repr = Self::#build_fn(self, &mut ctxt);
                unsafe { ctxt.add(repr, None); }
            }
            fn layout(&self, _: Layouter) -> Layout {
                Layout::Pass
            }
        }),
    })
}
