use inflector::Inflector;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use std::collections::{HashMap, HashSet};
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::token::{Comma, Paren};
use syn::{
    braced, parenthesized, parse_quote, Attribute, Error, Expr, Fields, GenericArgument,
    GenericParam, Generics, Ident, ItemStruct, Lit, LitStr, Path, PathArguments, Token, Type,
    Visibility,
};

struct PartField(Ident, Type);

impl Parse for PartField {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let ident = input.parse()?;
        let _: Token![:] = input.parse()?;
        let ty = input.parse()?;
        Ok(Self(ident, ty))
    }
}
impl ToTokens for PartField {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.0;
        let ty = &self.1;
        tokens.extend(quote! { #name: #ty });
    }
}

struct PartSpec {
    name: Ident,
    fields: Punctuated<PartField, Token![,]>,
}

impl Parse for PartSpec {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let name = format_ident!("Unnamed");
        let fields = input.parse_terminated(PartField::parse)?;
        Ok(Self { name, fields })
    }
}

impl ToTokens for PartSpec {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name;
        let fields = &self.fields;
        let content = quote! {
            struct #name {
                #fields
            }
        };
        tokens.extend(content);
    }
}

// Accepts a type which represents an RC and returns its target type. If the inputted type does not
// represent a RC, a compile error will be raised when the type is used.
fn rc_type_as_target(ty: &Type) -> Type {
    if let Type::Path(path) = ty {
        if path.qself.is_some() {
            // Return default.
        } else if let Some(segment) = path.path.segments.last() {
            if segment.ident == "Rc" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(arg)) = args.args.first() {
                        return arg.clone();
                    }
                }
            }
        }
    }
    // This way it also works with custom type aliases.
    parse_quote! { <#ty as ::core::ops::Deref>::Target }
}

struct WidgetInfo {
    vis: Visibility,
    name: Ident,
    parents: PartSpec,
    parent_types: Vec<Type>,
    children: PartSpec,
    state: PartSpec,
}

impl Parse for WidgetInfo {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let vis = input.parse()?;
        let name = input.parse()?;
        let mut parents: PartSpec = parse_quote! {};
        let mut children: PartSpec = parse_quote! {};
        let mut state: PartSpec = parse_quote! {};
        while !input.is_empty() {
            let kind: Ident = input.parse()?;
            if kind == "Parents" {
                let content;
                braced!(content in input);
                parents = content.parse()?;
            } else if kind == "Children" {
                let content;
                braced!(content in input);
                children = content.parse()?;
            } else if kind == "State" {
                let content;
                braced!(content in input);
                state = content.parse()?;
            } else {
                return Err(Error::new_spanned(
                    kind,
                    "Expected Parents, Children, or State.",
                ));
            }
        }
        let gui_interface = quote! { crate::GuiInterface<crate::PlaceholderGuiState> };
        let parent_types: Vec<_> = parents
            .fields
            .iter()
            .map(|PartField(_name, ty)| rc_type_as_target(ty))
            .collect();
        parents.fields.push(PartField(
            parse_quote! { gui },
            parse_quote! { ::std::rc::Rc<#gui_interface> },
        ));
        state.fields.push(PartField(
            parse_quote! { pos },
            parse_quote! { crate::Pos2D },
        ));
        parents.name = format_ident!("{}Parents", name);
        children.name = format_ident!("{}Children", name);
        state.name = format_ident!("{}State", name);
        assert!(input.is_empty());
        Ok(Self {
            vis,
            name,
            parents,
            parent_types,
            children,
            state,
        })
    }
}

impl ToTokens for WidgetInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let vis = &self.vis;
        let name = &self.name;
        let parents = &self.parents;
        let parents_type = &parents.name;
        let children = &self.children;
        let children_type = &children.name;
        let child_names: Vec<_> = self
            .children
            .fields
            .iter()
            .map(|PartField(name, _ty)| name)
            .collect();
        let state = &self.state;
        let state_type = &state.name;
        let parent_trait_name = format_ident!("{}Parent", name);
        let mut parent_names: Vec<_> = self
            .parents
            .fields
            .iter()
            .map(|PartField(name, _ty)| name)
            .collect();
        parent_names.pop().unwrap(); // We will deal with the GUI parent manually.
        let parent_types = &self.parent_types;

        let gui_interface_provider =
            quote! { crate::GuiInterfaceProvider<crate::PlaceholderGuiState> };
        let widget_provider_bounds = quote! {
            #gui_interface_provider +
            #(crate::WidgetProvider<crate::PlaceholderRenderer, ::std::rc::Rc<#parent_types>>)+*
        };
        let renderer = quote! { crate::PlaceholderRenderer };
        let dyn_mouse_behavior = quote! {
            ::core::option::Option<::std::boxed::Box<dyn crate::MouseBehavior>>
        };
        let self_ptr = quote! { ::std::rc::Rc<#name> };
        let ref_cell = quote! { ::std::cell::RefCell };
        let default = quote! { ::core::default::Default::default() };

        let content = quote! {
            #parents
            #[derive(Default)]
            #children
            #state

            #vis struct #name {
                parents: #parents_type,
                children: #ref_cell<#children_type>,
                state: #ref_cell<#state_type>,
            }

            #vis trait #parent_trait_name: #widget_provider_bounds { }
            impl<T> #parent_trait_name for T where T: #widget_provider_bounds { }

            impl #parents_type {
                fn new<P: #parent_trait_name>(parent: &P) -> Self {
                    Self {
                        gui: parent.provide_gui_interface(),
                        #(#parent_names: parent.provide()),*
                    }
                }
            }

            impl #name {
                fn create<P: #parent_trait_name>(parent: &P, initial_state: #state_type) -> Self {
                    Self {
                        parents: #parents_type::new(parent),
                        children: #ref_cell::new(#default),
                        state: #ref_cell::new(initial_state),
                    }
                }

                fn draw_children(self: &::std::rc::Rc<Self>, renderer: &mut #renderer) {
                    let children = self.children.borrow();
                    #(
                        for child in &children.#child_names {
                            child.draw(renderer);
                        }
                    )*
                }
            }
            #(
                impl crate::WidgetProvider<#renderer, ::std::rc::Rc<#parent_types>> for #self_ptr {
                    fn provide(&self) -> ::std::rc::Rc<#parent_types> {
                        ::std::rc::Rc::clone(&self.parents.#parent_names)
                    }
                }
            )*

            impl crate::Widget<#renderer> for #self_ptr {
                fn get_pos(&self) -> crate::Pos2D {
                    self.state.borrow().pos
                }

                fn get_mouse_behavior(&self, pos: crate::Pos2D) -> #dyn_mouse_behavior {
                    #name::get_mouse_behavior(self, pos)
                }

                fn draw(&self, renderer: &mut #renderer) {
                    renderer.push_state();
                    renderer.translate(self.get_pos());

                    #name::draw(self, renderer);

                    renderer.pop_state();
                }

                fn on_removed(&self) {
                    (*self.children.borrow_mut()) = #default;
                }
            }
        };
        tokens.extend(content);
    }
}

#[proc_macro]
pub fn widget(body: TokenStream) -> TokenStream {
    let info: WidgetInfo = syn::parse_macro_input!(body);
    (quote! { #info }).into()
}
