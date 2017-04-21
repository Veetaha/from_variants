use syn;
use quote::{ToTokens, Tokens};

use prelude::Bindings;

/// The generic type parameter used when `into` conversions are requested.
const INTO_GENERIC: &'static str = "INTO";

/// A view of data which can generate a `From<T> for Target` impl block.
#[derive(Debug, Clone)]
pub struct FromImpl<'a> {
    /// The set of library bindings to generate against (core or std).
    pub bindings: Bindings,
    
    /// The generics of the target enum.
    pub generics: &'a syn::Generics,
    
    /// The identifier of the target enum.
    pub target_ident: &'a syn::Ident,
    
    /// The identifier of the target variant.
    pub variant_ident: &'a syn::Ident,
    
    /// The type of the target variant.
    pub variant_ty: &'a syn::Ty,
    
    /// Whether or not this impl should use an `Into` conversion.
    pub into: bool,
}

impl<'a> ToTokens for FromImpl<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let from_trait = self.bindings.from_trait();
        let target_ident = &self.target_ident;
        let variant_ident = &self.variant_ident;
        let variant_ty = &self.variant_ty;
        let doc_comment = format!(include_str!("impl_doc.md"), variant = variant_ident);        
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        
        if self.into {
            let mut altered_generics: syn::Generics = self.generics.clone();
            altered_generics.ty_params.push(generate_into_ty_param(self.bindings.into_trait(), variant_ty));
            let (i, _, _) = altered_generics.split_for_impl();
            let into_variant = syn::parse_type(INTO_GENERIC).unwrap();
            
            tokens.append(quote!(
                #[doc = #doc_comment]
                impl #i #from_trait<#into_variant> for #target_ident #ty_generics
                    #where_clause {
                    fn from(v: #into_variant) -> Self {
                        #target_ident::#variant_ident(v.into())
                    }
                }
            ))
        } else {
            tokens.append(quote!(
                #[doc = #doc_comment]
                impl #impl_generics #from_trait<#variant_ty> for #target_ident #ty_generics
                    #where_clause {
                    fn from(v: #variant_ty) -> Self {
                        #target_ident::#variant_ident(v)
                    }
                }
            ))
        }
    }
}

fn generate_into_ty_param(into: syn::Path, variant_ty: &syn::Ty) -> syn::TyParam {
    syn::TyParam {
        ident: syn::Ident::new("INTO"),
        attrs: vec![],
        bounds: vec![syn::parse_ty_param_bound(quote!(#into<#variant_ty>).as_str()).unwrap()],
        default: None,
    }
}


macro_rules! default_from_impl {
    () => (
        {
            use syn;
            FromImpl {
                bindings: Default::default(),
                generics: &Default::default(),
                target_ident: &syn::Ident::new("Foo"),
                variant_ident: &syn::Ident::new("Bar"),
                variant_ty: &syn::parse_type("String").expect("default_from_impl should produce valid type"),
                into: false,
            }
        }
    )
}

#[cfg(test)]
mod tests {
    use syn;
    use super::FromImpl;
    
    #[test]
    fn simple() {
        let fi = default_from_impl!();
        assert_eq!(quote!(#fi), quote!(
            #[doc = "Convert into a `Bar` variant."]
            impl ::std::convert::From<String> for Foo {
                fn from(v: String) -> Self {
                    Foo::Bar(v)
                }
            }
        ));
    }
    
    #[test]
    fn lifetime() {
        let mut generics = syn::Generics::default();
        generics.lifetimes.push(syn::LifetimeDef::new("'a"));
        
        let ty = syn::parse_type("&'a str").unwrap();
        
        let mut fi = default_from_impl!();
        fi.variant_ty = &ty;
        fi.generics = &generics;
        
        assert_eq!(quote!(#fi), quote!(
            #[doc = "Convert into a `Bar` variant."]
            impl<'a> ::std::convert::From<&'a str> for Foo<'a> {
                fn from(v: &'a str) -> Self {
                    Foo::Bar(v)
                }
            }
        ));
    }
    
    #[test]
    fn into() {
        let mut fi = default_from_impl!();
        fi.into = true;
        
        assert_eq!(quote!(#fi), quote!(
            #[doc = "Convert into a `Bar` variant."]
            impl<INTO: ::std::convert::Into<String> > ::std::convert::From<INTO> for Foo {
                fn from(v: INTO) -> Self {
                    Foo::Bar(v.into())
                }
            }
        ));
    }
    
    #[test]
    fn into_generics() {
        let mut generics: syn::Generics = syn::Generics::default();
        let ty = syn::parse_type("Vec<T>").unwrap();
        generics.ty_params.push(syn::Ident::new("T").into());
        let mut fi = default_from_impl!();
        fi.variant_ty = &ty;
        fi.generics = &generics;
        fi.into = true;
        
        assert_eq!(quote!(#fi), quote!(
            #[doc = "Convert into a `Bar` variant."]
            impl<T, INTO: ::std::convert::Into<Vec<T> > > ::std::convert::From<INTO> for Foo<T> {
                fn from(v: INTO) -> Self {
                    Foo::Bar(v.into())
                }
            }
        ));
    }
}