use std::borrow::Borrow;

// An inefficient and limited but easy to use YAML representation.
#[derive(Default, Debug)]
pub struct YamlNode {
    pub name: String,
    pub full_name: String,
    pub value: String,
    pub children: Vec<Box<YamlNode>>,
}

impl YamlNode {
    pub fn unique_child(&self, child_name: &str) -> Result<&YamlNode, String> {
        let mut result = None;
        for child in &self.children {
            if child.name == child_name {
                if result.is_some() {
                    return Err(format!(
                        "ERROR: {} contains a duplicate entry for {}",
                        self.full_name, child_name
                    ));
                }
                result = Some(child.borrow());
            }
        }

        result.ok_or(format!(
            "ERROR: {} is missing an entry named {}.",
            self.full_name, child_name
        ))
    }

    pub fn f32(&self) -> Result<f32, String> {
        self.value.trim().parse().map_err(|_| {
            format!(
                "ERROR: The value of {} is not a valid decimal number",
                self.full_name
            )
        })
    }

    pub fn i32(&self) -> Result<i32, String> {
        self.value.trim().parse().map_err(|_| {
            format!(
                "ERROR: The value of {} is not a valid integer",
                self.full_name
            )
        })
    }
}

pub fn parse_yaml(input: &str, filename: &str) -> Result<YamlNode, String> {
    let mut stack = vec![YamlNode {
        name: filename.to_owned(),
        full_name: filename.to_owned(),
        value: "".to_owned(),
        children: Vec::new(),
    }];
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
