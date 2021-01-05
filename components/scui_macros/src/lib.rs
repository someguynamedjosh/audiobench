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
        let gui_interface = quote! { ::scui::GuiInterface<crate::scui_config::GuiState, crate::scui_config::DropTarget> };
        let parent_types: Vec<_> = parents
            .fields
            .iter()
            .map(|PartField(_name, ty)| rc_type_as_target(ty))
            .collect();
        parents.fields.push(PartField(
            parse_quote! { gui },
            parse_quote! { ::std::rc::Rc<#gui_interface> },
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

        let gui_state = quote! { crate::scui_config::GuiState };
        let drop_target = quote! { crate::scui_config::DropTarget };
        let gui_interface_provider = quote! { ::scui::GuiInterfaceProvider<#gui_state, #drop_target> };
        let renderer = quote! { crate::scui_config::Renderer };
        let widget_provider_bounds = quote! {
            #gui_interface_provider +
            #(::scui::WidgetProvider<#renderer, #drop_target, ::std::rc::Rc<#parent_types>>)+*
        };
        let self_ptr = quote! { ::std::rc::Rc<#name> };
        let ref_cell = quote! { ::std::cell::RefCell };
        let default = quote! { ::core::default::Default::default() };
        let widget_trait_turbo = quote! { ::scui::Widget::<#renderer, #drop_target> };
        let widget_impl_trait = quote! { ::scui::WidgetImpl<#renderer, #drop_target> };

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

                /// Returns `Some()` if any of the children of this widget have a mouse behavior at 
                /// the given position (as indicated by calling their `get_mouse_behavior` method.) 
                /// Returns `None` if no children provided a mouse behavior.
                fn get_mouse_behavior_children(self: &::std::rc::Rc<Self>, pos: ::scui::Vec2D, mods: &::scui::MouseMods) -> ::scui::MaybeMouseBehavior<#drop_target> {
                    let children = self.children.borrow();
                    #(
                        for child in &children.#child_names {
                            if let ::std::option::Option::Some(v) = #widget_trait_turbo::get_mouse_behavior(child, pos, mods) {
                                return Some(v);
                            }
                        }
                    )*
                    ::std::option::Option::None
                }

                /// Returns `Some()` if any of the children of this widget have a drop target at the
                /// given position (as indicated by calling their `get_drop_target` method.) Returns
                /// `None` if no children provided a drop target.
                fn get_drop_target_children(self: &::std::rc::Rc<Self>, pos: ::scui::Vec2D) -> ::std::option::Option<#drop_target> {
                    let children = self.children.borrow();
                    #(
                        for child in &children.#child_names {
                            if let ::std::option::Option::Some(v) = #widget_trait_turbo::get_drop_target(child, pos) {
                                return Some(v);
                            }
                        }
                    )*
                    ::std::option::Option::None
                }


                /// Calls the `on_scroll` method of each child of this widget, returning `Some` if any
                /// of the children returned Some and `None` if not. (Some children may not have their
                /// `on_scroll` called if an earlier child returns `Some`.)
                fn on_scroll_children(self: &::std::rc::Rc<Self>, pos: ::scui::Vec2D, delta: f32) -> ::std::option::Option<()> {
                    let children = self.children.borrow();
                    #(
                        for child in &children.#child_names {
                            if let ::std::option::Option::Some(v) = #widget_trait_turbo::on_scroll(child, pos, delta) {
                                return Some(v);
                            }
                        }
                    )*
                    ::std::option::Option::None
                }

                /// Calls the `on_hover` method of each child of this widget, returning `Some` if any
                /// of the children returned Some and `None` if not. (Some children may not have their
                /// `on_hover` called if an earlier child returns `Some`.)
                fn on_hover_children(self: &::std::rc::Rc<Self>, pos: ::scui::Vec2D) -> ::std::option::Option<()> {
                    let children = self.children.borrow();
                    #(
                        for child in &children.#child_names {
                            if let ::std::option::Option::Some(v) = #widget_trait_turbo::on_hover(child, pos) {
                                return Some(v);
                            }
                        }
                    )*
                    ::std::option::Option::None
                }

                /// Calls the `draw` method of all children of this widget.
                fn draw_children(self: &::std::rc::Rc<Self>, renderer: &mut #renderer) {
                    let children = self.children.borrow();
                    #(
                        for child in &children.#child_names {
                            #widget_trait_turbo::draw(child, renderer);
                        }
                    )*
                }

                fn with_gui_state<R>(self: &::std::rc::Rc<Self>, func: impl FnOnce(&#gui_state) -> R) -> R {
                    func(&*self.parents.gui.state.borrow())
                }

                fn with_gui_state_mut<R>(self: &::std::rc::Rc<Self>, func: impl FnOnce(&mut #gui_state) -> R) -> R {
                    func(&mut *self.parents.gui.state.borrow_mut())
                }
            }
            impl #gui_interface_provider for #self_ptr {
                fn provide_gui_interface(&self) -> ::std::rc::Rc<::scui::GuiInterface<#gui_state, #drop_target>> {
                    ::std::rc::Rc::clone(&self.parents.gui)
                }
            }
            #(
                impl ::scui::WidgetProvider<#renderer, #drop_target, ::std::rc::Rc<#parent_types>> for #self_ptr {
                    fn provide(&self) -> ::std::rc::Rc<#parent_types> {
                        ::std::rc::Rc::clone(&self.parents.#parent_names)
                    }
                }
            )*
            impl ::scui::WidgetProvider<#renderer, #drop_target, #self_ptr> for #self_ptr {
                fn provide(&self) -> #self_ptr {
                    ::std::rc::Rc::clone(self)
                }
            }

            impl ::scui::Widget<#renderer, #drop_target> for #self_ptr {
                fn get_pos(&self) -> ::scui::Vec2D {
                    <#name as #widget_impl_trait>::get_pos_impl(self)
                }

                fn get_size(&self) -> ::scui::Vec2D {
                    <#name as #widget_impl_trait>::get_size_impl(self)
                }

                fn get_mouse_behavior(&self, pos: ::scui::Vec2D, mods: &::scui::MouseMods) -> ::scui::MaybeMouseBehavior<#drop_target> {
                    let pos = pos - self.get_pos();
                    if !pos.inside(self.get_size()) {
                        return None;
                    }
                    <#name as #widget_impl_trait>::get_mouse_behavior_impl(self, pos, mods)
                }

                fn get_drop_target(&self, pos: ::scui::Vec2D) -> ::std::option::Option::<#drop_target> {
                    let pos = pos - self.get_pos();
                    if !pos.inside(self.get_size()) {
                        return None;
                    }
                    <#name as #widget_impl_trait>::get_drop_target_impl(self, pos)
                }

                fn on_scroll(&self, pos: ::scui::Vec2D, delta: f32) -> ::std::option::Option::<()> {
                    let pos = pos - self.get_pos();
                    if !pos.inside(self.get_size()) {
                        return None;
                    }
                    <#name as #widget_impl_trait>::on_scroll_impl(self, pos, delta)
                }

                fn on_hover(&self, pos: ::scui::Vec2D) -> ::std::option::Option::<()> {
                    let pos = pos - self.get_pos();
                    if !pos.inside(self.get_size()) {
                        return None;
                    }
                    <#name as #widget_impl_trait>::on_hover_impl(self, pos)
                }

                fn draw(&self, renderer: &mut #renderer) {
                    use ::scui::Renderer;

                    renderer.push_state();
                    renderer.translate(#widget_trait_turbo::get_pos(self));

                    <#name as #widget_impl_trait>::draw_impl(self, renderer);

                    renderer.pop_state();
                }

                fn on_removed(&self) {
                    (*self.children.borrow_mut()) = #default;
                }
            }

            impl ::scui::WidgetImplDefaults<#renderer, #drop_target> for #name {
                fn get_mouse_behavior_default(self: &#self_ptr, pos: ::scui::Vec2D, mods: &::scui::MouseMods) -> ::scui::MaybeMouseBehavior<#drop_target> {
                    #name::get_mouse_behavior_children(self, pos, mods)
                }

                fn get_drop_target_default(self: &#self_ptr, pos: ::scui::Vec2D) -> ::std::option::Option<#drop_target> {
                    #name::get_drop_target_children(self, pos)
                }

                fn on_scroll_default(self: &#self_ptr, pos: ::scui::Vec2D, delta: f32) -> ::std::option::Option<()> {
                    #name::on_scroll_children(self, pos, delta)
                }

                fn on_hover_default(self: &#self_ptr, pos: ::scui::Vec2D) -> ::std::option::Option<()> {
                    #name::on_hover_children(self, pos)
                }

                fn draw_default(self: &#self_ptr, renderer: &mut #renderer) {
                    #name::draw_children(self, renderer);
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
