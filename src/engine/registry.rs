use crate::engine::{Control, GuiOutline, Module, WidgetOutline};
use std::collections::HashMap;

fn create_module_prototype_from_yaml(text: &str) -> Result<Module, String> {
    unimplemented!();
}

// An inefficient and limited but easy to use YAML representation.
#[derive(Default, Debug)]
struct YamlNode {
    pub name: String,
    pub full_name: String,
    pub value: String,
    pub children: Vec<Box<YamlNode>>,
}

fn parse_yaml(input: &str, filename: &str) -> Result<YamlNode, String> {
    let mut stack = vec![
        YamlNode {
            name: filename.to_owned(),
            full_name: filename.to_owned(),
            value: "".to_owned(),
            children: Vec::new(),
        },
    ];
    enum ParseMode {
        Name,
        Value,
        Indent,
    }
    use ParseMode::*;

    let mut mode = Indent;
    let mut current_indent_level = 0;
    let mut pos = (1, 0);
    macro_rules! error {
        ($message:expr) => {
            return Err(format!(
                concat!($message, ": {}:{}:{}"),
                filename, pos.0, pos.1
            ));
        };
    }

    for c in input.chars() {
        if c == '\r' {
            continue;
        }
        if c == '\n' {
            pos.0 += 1;
            pos.1 = 0;
        } else {
            pos.1 += 1;
        }
        match mode {
            Indent => {
                if c == ' ' {
                    if pos.1 > current_indent_level {
                        error!("Too much indentation");
                    }
                } else if c == '\n' {
                    error!("Unexpected newline");
                } else {
                    if (pos.1 - 1) % 2 != 0 {
                        error!("Wrong amount of indentation");
                    }
                    let indent = (pos.1 - 1) / 2;
                    let deindent = current_indent_level / 2 - indent;
                    for _ in 0..deindent {
                        // Stack problems should be caught earlier as indentation problems.
                        let last = stack.pop().unwrap();
                        let top_index = stack.len() - 1;
                        stack[top_index].children.push(Box::new(last));
                        current_indent_level -= 2;
                    }
                    mode = Name;
                    let mut new: YamlNode = Default::default();
                    let top_index = stack.len() - 1;
                    new.name.push(c);
                    new.full_name = stack[top_index].full_name.clone();
                    stack.push(new);
                }
            }
            Name => {
                let top_index = stack.len() - 1;
                let mut top = &mut stack[top_index];
                if c == '\n' {
                    error!("Unexpected newline");
                } else if c == ':' {
                    top.name = top.name.trim().to_owned();
                    top.full_name = format!("{}.{}", top.full_name, top.name);
                    mode = Value;
                } else {
                    top.name.push(c);
                }
            }
            Value => {
                let top_index = stack.len() - 1;
                let mut top = &mut stack[top_index];
                if c == '\n' {
                    top.value = top.value.trim().to_owned();
                    current_indent_level += 2;
                    mode = Indent;
                } else {
                    top.value.push(c);
                }
            }
        }
    }
    for _ in 1..stack.len() {
        let last = stack.pop().unwrap();
        let top_index = stack.len() - 1;
        stack[top_index].children.push(Box::new(last));
    }
    Ok(stack.pop().unwrap())
}

pub struct Registry {}

impl Registry {
    pub fn new() -> Self {
        let yaml = std::include_str!("../../modules/note_input.yaml");
        println!("{:#?}", parse_yaml(yaml, "embedded"));
        Self {}
    }
}
