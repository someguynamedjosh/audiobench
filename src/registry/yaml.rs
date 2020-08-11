use std::borrow::Borrow;
use std::fmt::Display;
use std::str::FromStr;

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

    /// Returns `Ok` if `self.value` can be parsed as a `T`. This is equivalent to doing
    /// `self.value.trim().parse()`. If the value cannot be parsed, a human readable error is
    /// returned along the lines of "ERROR: The value of {path.to.node} is not a valid {type}".
    pub fn parse<T: FromStr>(&self) -> Result<T, String> {
        self.value.trim().parse().map_err(|_| {
            format!(
                "ERROR: The value of {} is not a valid {}",
                self.full_name,
                std::any::type_name::<T>()
            )
        })
    }

    /// Like `parse()` but uses a custom parse function instead of `str::parse`. Any error returned
    /// by the custom parser will be wrapped with another error specifying the path of this node,
    /// so you do not have to add that information yourself.
    pub fn custom_parse<T>(
        &self,
        parser: impl FnOnce(&str) -> Result<T, String>,
    ) -> Result<T, String> {
        parser(self.value.trim()).map_err(|err| {
            format!(
                "ERROR: The value of {} is not valid, caused by:\n",
                self.full_name
            )
        })
    }

    /// Like `parse()` but returns an error if the value is outside the specified range. The min/max
    /// bounds are inclusive. You can pass None for a bound if the value should be unbounded in that
    /// direction.
    pub fn parse_ranged<T: FromStr + PartialOrd + Display>(
        &self,
        min: Option<T>,
        max: Option<T>,
    ) -> Result<T, String> {
        let value = self.parse()?;
        if let Some(min) = min {
            if value < min {
                return Err(format!(
                    "ERROR: The value of {} is less than the minimum value of '{}'",
                    self.full_name, min
                ));
            }
        }
        if let Some(max) = max {
            if value > max {
                return Err(format!(
                    "ERROR: The value of {} is greater than the maximum value of '{}'",
                    self.full_name, max
                ));
            }
        }
        Ok(value)
    }

    /// Returns the index of the item in `possible_values` which matches `self.value.trim()`. If
    /// there is no matching item, a human readable error is returned containing the location of 
    /// the errror and a list of legal values. E.G.: "ERROR: The value of path.to.node is not one
    /// of 'foo', 'bar', 'baz',". I'm too lazy to get rid of the last comma.
    pub fn enumerated(&self, possible_values: &[&str]) -> Result<usize, String> {
        let self_value = self.value.trim();
        for (index, value) in possible_values.iter().enumerate() {
            if self_value == *value {
                return Ok(index);
            }
        }
        let mut err = format!("ERROR: The value of {} is not one of", self.full_name);
        for value in possible_values {
            err.push_str(&format!(" '{}',", value));
        }
        Err(err)
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
