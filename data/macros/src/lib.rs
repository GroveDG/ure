use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Block, FnArg, Ident, ItemFn, Pat, PatType, Signature, Token, Type, Visibility, parenthesized,
    parse::Parse,
    parse_macro_input,
    punctuated::Punctuated,
    token::{Comma, Paren},
};

struct Arg {
    ident: Ident,
    colon: Token![:],
    ty: Type,
}
impl Parse for Arg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Arg {
            ident: input.parse()?,
            colon: input.parse()?,
            ty: input.parse()?,
        })
    }
}

mod kw {
    use syn::custom_keyword;

    custom_keyword!(args);
    custom_keyword!(get);
    custom_keyword!(extract);
}

enum ArgsKeyword {
    Args(kw::args),
    Get(kw::get),
    Extract(kw::extract),
}
impl Parse for ArgsKeyword {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::args) {
            Ok(Self::Args(input.parse()?))
        } else if lookahead.peek(kw::get) {
            Ok(Self::Get(input.parse()?))
        } else if lookahead.peek(kw::extract) {
            Ok(Self::Extract(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

struct ArgGroup {
    key: ArgsKeyword,
    paren: Paren,
    args: Punctuated<Arg, Token![,]>,
}
impl ArgGroup {
    fn split(&self) -> (Vec<&Ident>, Vec<&Type>) {
        let mut idents = Vec::new();
        let mut types = Vec::new();

        for arg in self.args.iter() {
            idents.push(&arg.ident);
            types.push(&arg.ty);
        }

        (idents, types)
    }
}
impl Parse for ArgGroup {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(ArgGroup {
            key: input.parse()?,
            paren: parenthesized!(content in input),
            args: content.parse_terminated(Arg::parse, Token![,])?,
        })
    }
}

struct Method {
    vis: Visibility,
    ident: Ident,
    extract: Option<ArgGroup>,
    get: Option<ArgGroup>,
    args: Option<ArgGroup>,
    block: Block,
}
impl Parse for Method {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let vis = input.parse()?;
        _ = input.parse::<Token![fn]>()?;
        let ident = input.parse()?;
        _ = parenthesized!(content in input);
        let extract = if content.peek(kw::extract) {
            Some(content.parse()?)
        } else {
            None
        };
        let get = if content.peek(kw::get) {
            Some(content.parse()?)
        } else {
            None
        };
        let args = if content.peek(kw::args) {
            Some(content.parse()?)
        } else {
            None
        };
        // if !content.is_empty() {
        //     return Err(content.error("expected end of args"));
        // }
        Ok(Method {
            vis,
            ident,
            extract,
            get,
            args,
            block: input.parse()?,
        })
    }
}

#[proc_macro_attribute]
pub fn method(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_input = parse_macro_input!(item as Method);

    let Method {
        vis,
        ident,
        extract,
        get,
        args,
        block,
    } = item_input;

    let ident_const: Ident = Ident::new(&ident.to_string().to_uppercase(), ident.span());

    let (extract_idents, extract_types) = extract
        .as_ref()
        .map(ArgGroup::split)
        .unwrap_or((Vec::new(), Vec::new()));
    let (get_idents, get_types) = get
        .as_ref()
        .map(ArgGroup::split)
        .unwrap_or((Vec::new(), Vec::new()));
    let (args_idents, args_types) = args
        .as_ref()
        .map(ArgGroup::split)
        .unwrap_or((Vec::new(), Vec::new()));

    quote! {
        #vis const #ident_const: ure_data::Method<#(#args_types),*> = ure_data::Method::<#(#args_types),*>::new(
            |components, args| {
                #(
                let #extract_idents = components.remove(&<#extract_types as ure_data::Component>::ID).unwrap();
                )*
                let components = <(#(#get_types),*)>::retrieve_mut(components).unwrap();
                let (#(#get_idents),*) = components;
                #ident(#(#extract_idents,)* #(#get_idents,)* args);
                #(
                components.insert(<#extract_types as ure_data::Component>::ID, #extract_idents);
                )*
            },
            stringify!(#ident),
            <(#(#get_types,)* #(#extract_types,)*)>::IDS,
        );

        #vis fn #ident (
            #(#extract_idents: &mut <#extract_types as ure_data::Component>::Container,)*
            #(#get_idents: <<#get_types as ure_data::Component>::Container as ure_data::Container>::Mut<'_>,)*
            #(#args_idents: #args_types,)*
        )
        #block
    }
    .into()
}
