extern crate syn;
extern crate quote;
extern crate proc_macro2;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input,parse, punctuated::Punctuated};
use syn::Token;

struct Pair{
    ident: syn::Ident,
    ty: syn::Type
}

impl parse::Parse for Pair {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;
        let _ = input.parse::<Token!(:)>()?;
        let ty = input.parse()?;
        Ok(Self{
            ident,
            ty
        })
    }
}

struct Remainder{
    args:Punctuated<Pair,Token!(,)>
}
impl parse::Parse for Remainder {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        Ok(Remainder{
            args:input.parse_terminated(Pair::parse, Token!(,))?
        })
    }
}
enum LeadingArg{
    ImutRef(Token!(&),Token!(self), Token!(,), Remainder),
    MutRef(Token!(&), Token!(mut), Token!(self),Token!(,), Remainder),
    Owned(Token!(self),Token!(,),Remainder),
    None(Remainder)
}
impl parse::Parse for LeadingArg {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token!(&)) {
            let and:Token!(&) = input.parse()?;
            if input.peek(Token!(mut)){
                let mut_t:Token!(mut) = input.parse()?;
                if input.peek(Token!(self)){
                    let self_t:Token!(self) = input.parse()?;
                    Ok(LeadingArg::MutRef(and,mut_t,self_t,input.parse()?,input.parse()?))
                }
                else{
                    Ok(LeadingArg::None(input.parse()?))
                }
            }
            else{
                if input.peek(Token!(self)){
                    let self_t:Token!(self) = input.parse()?;
                    Ok(LeadingArg::ImutRef(and,self_t,input.parse()?,input.parse()?))
                }
                else{
                    Ok(LeadingArg::None(input.parse()?))
                }

            }
        }
        else{
                if input.peek(Token!(self)){
                    let self_t:Token!(self) = input.parse()?;
                    Ok(LeadingArg::Owned(self_t,input.parse()?,input.parse()?))
                }
                else{
                    Ok(LeadingArg::None(input.parse()?))
                }

        }
    }
}
struct CompleteArg{
    #[allow(dead_code)]
    paren:syn::token::Paren,
    args:LeadingArg
}
impl parse::Parse for CompleteArg{
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(CompleteArg{
            paren: syn::parenthesized!(content in input),
            args: content.parse()?
        })
    }
}
#[proc_macro]
pub fn transform_self_arg(tokens: TokenStream)->TokenStream{
    let CompleteArg{paren:_,args:parsed} = parse_macro_input!(tokens as CompleteArg);
    match parsed {
        LeadingArg::None(rem) => {
            //very ineffiecient but does the job
            rem.args.into_iter().map(|pair|pair.ident.to_string()+":"+pair.ty.to_token_stream().to_string().as_str()).collect::<Vec<String>>().join(",").to_token_stream().into()
        },
        LeadingArg::ImutRef(_, _, _, rem) | LeadingArg::MutRef(_, _, _, _, rem) | LeadingArg::Owned(_, _, rem)=>{
            let stream = rem.args.into_iter().map(|pair|pair.ident.to_string()+":"+pair.ty.to_token_stream().to_string().as_str()).collect::<Vec<String>>();
            let mut prefix = quote!{__data: std::ptr::NonNull<()>};
            if stream.len() > 0 {
                prefix.extend(",".to_token_stream())
            }
            prefix.extend(stream.join(",").to_token_stream());
            //super hacky solution
            let mut prefix:Vec<_> = prefix.into_iter().collect();
            prefix.pop();
            let prefix:proc_macro2::TokenStream = prefix.into_iter().collect();
            //println!("{}",quote!{fn(#prefix)});
            quote!{fn(#prefix)}.into()
        }
    }
}
/// make the args of the func call in the body of the closure so it appropriately makes NonNull into as_ref, as_mut or to_owned
/// as needed
#[proc_macro]
pub fn construct_closure_body_args(tokens: TokenStream)->TokenStream{
    let CompleteArg{paren:_,args:parsed} = parse_macro_input!(tokens as CompleteArg);
    let args = match parsed {
        LeadingArg::None(rem) => {
            ("",strip_types(rem.args).map(|i|i.to_string()).collect::<Vec<String>>().join(","))
        },
        LeadingArg::ImutRef(_, _, _, rem)=>{
           ("__data.as_ref(),",strip_types(rem.args).map(|i|i.to_string()).collect::<Vec<String>>().join(",")) 
        },
        LeadingArg::MutRef(_, _, _, _, rem)=>{
            ("__data.as_mut()",strip_types(rem.args).map(|i|i.to_string()).collect::<Vec<String>>().join(","))
        },
        LeadingArg::Owned(_, _, rem)=>{
            ("__data.to_owned()",strip_types(rem.args).map(|i|i.to_string()).collect::<Vec<String>>().join(","))
        }
    };
    let mut pre = args.0.to_token_stream();
    pre.extend(args.1.to_token_stream());
    pre.into()
}

fn strip_types(args:impl IntoIterator<Item = Pair>)->std::iter::Map<impl std::iter::Iterator<Item = Pair>, impl FnMut(Pair)->syn::Ident>{
    args.into_iter().map(|p|p.ident)
}
