use inflector::Inflector;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parenthesized, Expr, Ident, Token};

#[derive(PartialEq)]
enum ConstructorItemType {
    RegistryRef,
    GridPos,
    GridSize,
    AutoconRef,
    StaticonRef,
    IntRange,
    FloatRange,
    String,
    StringList,
}

struct ConstructorItem {
    name: Ident,
    typ: ConstructorItemType,
}

impl ConstructorItem {
    pub fn get_outline_fields(&self) -> Vec<(Ident, TokenStream2)> {
        match self.typ {
            ConstructorItemType::RegistryRef => vec![],
            ConstructorItemType::AutoconRef => {
                vec![(format_ident!("{}_index", self.name), quote! {usize})]
            }
            ConstructorItemType::StaticonRef => {
                vec![(format_ident!("{}_index", self.name), quote! {usize})]
            }
            ConstructorItemType::IntRange => vec![(self.name.clone(), quote! { (i32, i32)})],
            ConstructorItemType::GridPos
            | ConstructorItemType::GridSize
            | ConstructorItemType::FloatRange => vec![(self.name.clone(), quote! { (f32, f32)})],
            ConstructorItemType::String => {
                vec![(self.name.clone(), quote! {::std::string::String})]
            }
            ConstructorItemType::StringList => vec![(
                self.name.clone(),
                quote! {::std::vec::Vec<::std::string::String>},
            )],
        }
    }

    pub fn create_from_yaml_code(&self) -> TokenStream2 {
        match self.typ {
            ConstructorItemType::RegistryRef => quote! {},
            ConstructorItemType::GridPos => {
                let name = self.name.clone();
                quote! {
                    let #name = (
                        crate::gui::constants::coord(yaml.unique_child("x")?.parse()?)
                            + crate::gui::constants::JACK_SIZE
                            + crate::gui::constants::MODULE_IO_WIDTH,
                        crate::gui::constants::coord(yaml.unique_child("y")?.parse()?),
                    );
                }
            }
            ConstructorItemType::GridSize => {
                let name = self.name.clone();
                quote! {
                    let #name = (
                        crate::gui::constants::grid(yaml.unique_child("w")?.parse()?),
                        crate::gui::constants::grid(yaml.unique_child("h")?.parse()?),
                    );
                }
            }
            ConstructorItemType::AutoconRef => {
                let name = self.name.clone();
                let name_name = format_ident!("{}_name", self.name);
                let index_name = format_ident!("{}_index", self.name);
                quote! {
                    let #name_name = yaml.unique_child(stringify!(#name))?.value.trim();
                    let #index_name = find_control_index(#name_name)?;
                }
            }
            ConstructorItemType::StaticonRef => {
                let name = self.name.clone();
                let name_name = format_ident!("{}_name", self.name);
                let index_name = format_ident!("{}_index", self.name);
                quote! {
                    let #name_name = yaml.unique_child(stringify!(#name))?.value.trim();
                    let #index_name = find_staticon_index(#name_name)?;
                }
            }
            ConstructorItemType::IntRange => {
                let name = self.name.clone();
                quote! {
                    let #name = (
                        yaml.unique_child("min")?.parse()?,
                        yaml.unique_child("max")?.parse()?,
                    );
                }
            }
            ConstructorItemType::FloatRange => {
                let name = self.name.clone();
                quote! {
                    let #name = (
                        yaml.unique_child("min")?.parse()?,
                        yaml.unique_child("max")?.parse()?,
                    );
                }
            }
            ConstructorItemType::String => {
                let name = self.name.clone();
                quote! {
                    let #name = yaml.unique_child(stringify!(#name))?.value.trim().to_owned();
                }
            }
            ConstructorItemType::StringList => {
                let name = self.name.clone();
                quote! {
                    let mut #name = Vec::new();
                    for child in &yaml.unique_child(stringify!(#name))?.children {
                        #name.push(child.name.clone());
                    }
                    if #name.len() < 2 {
                        return Err(format!(
                            concat!(
                                "ERROR: Invalid widget {}, caused by:\n",
                                "ERROR: ",
                                stringify!(#name),
                                " must have at least 2 options."
                            ),
                            &yaml.full_name
                        ));
                    }
                }
            }
        }
    }

    pub fn create_constructor_argument(&self) -> TokenStream2 {
        match self.typ {
            ConstructorItemType::RegistryRef => quote! { registry },
            ConstructorItemType::GridPos
            | ConstructorItemType::GridSize
            | ConstructorItemType::IntRange
            | ConstructorItemType::FloatRange
            | ConstructorItemType::String
            | ConstructorItemType::StringList => {
                let name = self.name.clone();
                quote! { self.#name.clone() }
            }
            ConstructorItemType::AutoconRef => {
                let name = format_ident!("{}_index", self.name);
                quote! { ::std::rc::Rc::clone(&controls[self.#name]) }
            }
            ConstructorItemType::StaticonRef => {
                let name = format_ident!("{}_index", self.name);
                quote! { ::std::rc::Rc::clone(&staticons[self.#name]) }
            }
        }
    }
}

impl Parse for ConstructorItem {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let type_name: Ident = input.parse()?;
        let typ = match &type_name.to_string()[..] {
            "RegistryRef" => ConstructorItemType::RegistryRef,
            "GridPos" => ConstructorItemType::GridPos,
            "GridSize" => ConstructorItemType::GridSize,
            "AutoconRef" => ConstructorItemType::AutoconRef,
            "StaticonRef" => ConstructorItemType::StaticonRef,
            "IntRange" => ConstructorItemType::IntRange,
            "FloatRange" => ConstructorItemType::FloatRange,
            "String" => ConstructorItemType::String,
            "StringList" => ConstructorItemType::StringList,
            _ => panic!(
                "{} is not a recognized constructor parameter type",
                type_name
            ),
        };
        Ok(Self { name, typ })
    }
}

struct ConstructorDescription {
    name: Ident,
    args: Punctuated<ConstructorItem, Token![,]>,
}

impl Parse for ConstructorDescription {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let name = input.parse()?;
        let arg_stream;
        parenthesized!(arg_stream in input);
        let args = arg_stream.parse_terminated(Parse::parse)?;
        Ok(Self { name, args })
    }
}

enum FeedbackDescription {
    Autocon,
    Custom { size: Expr },
}

impl Parse for FeedbackDescription {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let typ: Ident = input.parse()?;
        Ok(match &typ.to_string()[..] {
            "control" => Self::Autocon,
            "custom" => {
                let size_stream;
                parenthesized!(size_stream in input);
                Self::Custom {
                    size: size_stream.parse()?,
                }
            }
            _ => panic!("{} is not a valid feedback data mode", typ),
        })
    }
}

struct WidgetOutlineDescription {
    widget_struct_name: Option<Ident>,
    constructor_description: Option<ConstructorDescription>,
    ccdp_name: Option<Ident>,
    feedback_description: Option<FeedbackDescription>,
}

impl Parse for WidgetOutlineDescription {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let mut result = WidgetOutlineDescription {
            widget_struct_name: None,
            constructor_description: None,
            ccdp_name: None,
            feedback_description: None,
        };
        while !input.is_empty() {
            let name: Ident = input.parse()?;
            input.parse::<Token![:]>()?;
            match &name.to_string()[..] {
                "widget_struct" => result.widget_struct_name = Some(input.parse()?),
                "constructor" => result.constructor_description = Some(input.parse()?),
                "staticon_default_provider" => result.ccdp_name = Some(input.parse()?),
                "feedback" => result.feedback_description = Some(input.parse()?),
                _ => panic!("Unexpected identifier {}", name),
            }
            input.parse::<Token![,]>().ok(); // ignore because there might not be a trailing comma
        }
        Ok(result)
    }
}

#[proc_macro]
pub fn make_widget_outline(args: TokenStream) -> TokenStream {
    let WidgetOutlineDescription {
        widget_struct_name,
        constructor_description,
        ccdp_name,
        feedback_description,
    } = syn::parse_macro_input!(args);

    let widget_struct_name = widget_struct_name.expect("widget_struct not specified");
    let constructor_description = constructor_description.expect("constructor not specified");
    let outline_name = format_ident!("Generated{}Outline", widget_struct_name);

    let mut outline_fields = Vec::new();
    for constructor_arg in &constructor_description.args {
        outline_fields.append(&mut constructor_arg.get_outline_fields());
    }

    let feedback_requirement_code = match &feedback_description {
        None => quote! { crate::gui::module_widgets::FeedbackDataRequirement::None },
        Some(FeedbackDescription::Autocon) => {
            let control_name = constructor_description
                .args
                .iter()
                .find(|item| item.typ == ConstructorItemType::AutoconRef)
                .expect(
                    "feedback is set to control, but the constructor takes no control references!",
                )
                .name
                .clone();
            let control_name = format_ident!("{}_index", control_name);
            quote! {
                crate::gui::module_widgets::FeedbackDataRequirement::Autocon {
                    control_index: self.#control_name,
                }
            }
        }
        Some(FeedbackDescription::Custom { size }) => {
            outline_fields.push((
                format_ident!("feedback_name"),
                quote! {::std::string::String},
            ));
            quote! {
                crate::gui::module_widgets::FeedbackDataRequirement::Custom {
                    code_name: self.feedback_name.clone(),
                    size: #size
                }
            }
        }
    };

    let outline_field_names: Vec<_> = outline_fields
        .iter()
        .map(|(name, _typ)| name.clone())
        .collect();
    let mut field_from_yaml_code: Vec<_> = constructor_description
        .args
        .iter()
        .map(|arg| arg.create_from_yaml_code())
        .collect();
    if let Some(FeedbackDescription::Custom { .. }) = &feedback_description {
        field_from_yaml_code.push(quote! {
            let feedback_name = yaml.unique_child("feedback_name")?.value.trim().to_owned();
        });
    }
    let outline_fields: Vec<_> = outline_fields
        .into_iter()
        .map(|(name, typ)| {
            quote! {
                #name : #typ
            }
        })
        .collect();
    let ccdefault_code = if let Some(ccdp_name) = ccdp_name {
        quote! {
            let set_defaults: Vec<(usize, String)> = #widget_struct_name::#ccdp_name(&result, yaml)?;
            for (index, value) in set_defaults {
                if staticons[index] .borrow().value != "" {
                    return ::std::result::Result::Err(format!(
                        "ERROR: Multiple widgets controlling the same complex control {}.",
                        staticons[index].borrow().code_name
                    ));
                }
                staticons[index].borrow_mut().default = value.clone();
                staticons[index].borrow_mut().value = value;
            }
        }
    } else {
        quote! {}
    };

    let constructor_name = constructor_description.name.clone();
    let constructor_arg_values: Vec<_> = constructor_description
        .args
        .iter()
        .map(ConstructorItem::create_constructor_argument)
        .collect();

    TokenStream::from(quote! {
        #[derive(Clone, Debug)]
        pub struct #outline_name {
            #(#outline_fields),*
        }

        impl #outline_name {
            pub fn get_feedback_data_requirement(&self)
                -> crate::gui::module_widgets::FeedbackDataRequirement {
                #feedback_requirement_code
            }

            pub fn from_yaml(
                yaml: &crate::registry::yaml::YamlNode,
                controls: & ::std::vec::Vec<::std::rc::Rc<::std::cell::RefCell<crate::engine::parts::Autocon>>>,
                staticons: &mut ::std::vec::Vec<::std::rc::Rc<::std::cell::RefCell<crate::engine::parts::Staticon>>>,
            ) -> ::std::result::Result<#outline_name, ::std::string::String> {
                let find_control_index = |name: &str| {
                    controls
                        .iter()
                        .position(|item| &item.borrow().code_name == name)
                        .ok_or_else(|| {
                            format!(
                                "ERROR: Invalid widget {}, caused by:\nERROR: No control named {}.",
                                &yaml.full_name, name
                            )
                        })
                };
                let find_staticon_index = |name: &str| {
                    staticons
                        .iter()
                        .position(|item| &item.borrow().code_name == name)
                        .ok_or_else(|| {
                            format!(
                                "ERROR: Invalid widget {}, caused by:\nERROR: No complex control named {}.",
                                &yaml.full_name, name
                            )
                        })
                };

                #(#field_from_yaml_code)*

                let result = #outline_name {
                    #(#outline_field_names),*
                };
                #ccdefault_code
                ::std::result::Result::Ok(result)
            }

            pub fn instantiate(
                &self,
                registry: &crate::registry::Registry,
                controls: & ::std::vec::Vec<::std::rc::Rc<::std::cell::RefCell<crate::engine::parts::Autocon>>>,
                staticons: & ::std::vec::Vec<::std::rc::Rc<::std::cell::RefCell<crate::engine::parts::Staticon>>>,
            ) -> #widget_struct_name {
                #widget_struct_name::#constructor_name(#(#constructor_arg_values),*)
            }
        }
    })
}

struct ClassList {
    class_names: Punctuated<Ident, Token![,]>,
}

impl Parse for ClassList {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        Ok(Self {
            class_names: input.parse_terminated(Parse::parse)?,
        })
    }
}

#[proc_macro]
pub fn make_widget_outline_enum(args: TokenStream) -> TokenStream {
    let ClassList { class_names } = syn::parse_macro_input!(args);

    let enum_body: Vec<_> = class_names
        .iter()
        .map(|name| {
            let outline_struct_name = format_ident!("Generated{}Outline", name);
            quote! { #name(#outline_struct_name) }
        })
        .collect();
    let feedback_body: Vec<_> = class_names
        .iter()
        .map(|name| {
            quote! { Self::#name(outline) => outline.get_feedback_data_requirement() }
        })
        .collect();
    let from_yaml_body: Vec<_> = class_names
        .iter()
        .map(|name| {
            let snake_case_string = name.to_string().to_snake_case();
            let outline_struct_name = format_ident!("Generated{}Outline", name);
            quote! {
                #snake_case_string => Self::#name(#outline_struct_name::from_yaml(
                    yaml, controls, staticons
                )?)
            }
        })
        .collect();
    let instantiate_body: Vec<_> = class_names
        .iter()
        .map(|name| {
            quote! {
                Self::#name(outline) => Box::new(
                    outline.instantiate(registry, controls, staticons)
                )
            }
        })
        .collect();

    TokenStream::from(quote! {
        #[derive(Clone, Debug)]
        pub enum WidgetOutline {
            #(#enum_body),*
        }

        impl WidgetOutline {
            pub fn get_feedback_data_requirement(&self) -> crate::gui::module_widgets::FeedbackDataRequirement {
                match self {
                    #(#feedback_body),*
                }
            }

            pub fn from_yaml(
                yaml: &crate::registry::yaml::YamlNode,
                controls: & ::std::vec::Vec<::std::rc::Rc<::std::cell::RefCell<crate::engine::parts::Autocon>>>,
                staticons: &mut ::std::vec::Vec<::std::rc::Rc<::std::cell::RefCell<crate::engine::parts::Staticon>>>,
            ) -> ::std::result::Result<Self, ::std::string::String> {
                Ok(match &yaml.name[..] {
                    #(#from_yaml_body),*,
                    _ => {
                        return Err(format!(
                            "ERROR: Invalid widget {}, caused by:\nERROR: {} is not a valid widget type.",
                            &yaml.full_name, &yaml.name
                        ))
                    }
                })
            }

            pub fn instantiate(
                &self,
                registry: &crate::registry::Registry,
                controls: & ::std::vec::Vec<::std::rc::Rc<::std::cell::RefCell<crate::engine::parts::Autocon>>>,
                staticons: & ::std::vec::Vec<::std::rc::Rc<::std::cell::RefCell<crate::engine::parts::Staticon>>>,
            ) -> (::std::boxed::Box<dyn crate::gui::module_widgets::ModuleWidget>, usize) {
                (
                    match self {
                        #(#instantiate_body),*
                    },
                    self.get_feedback_data_requirement().size(),
                )
            }
        }
    })
}
