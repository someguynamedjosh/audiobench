use std::{fmt::Display, str::FromStr};
use yaml_rust::{Yaml, YamlLoader};

// An inefficient and limited but easy to use YAML representation.
#[scones::make_constructor]
#[derive(Clone, Debug)]
pub struct YamlNode {
    pub name: String,
    pub full_name: String,
    data: Yaml,
}

impl YamlNode {
    pub fn map_entry(&mut self, child_name: &str) -> Result<YamlNode, String> {
        let mut result = None;
        if let Yaml::Hash(map) = &mut self.data {
            for entry in map.entries() {
                if let Yaml::String(key) = entry.key() {
                    if key == child_name {
                        let full_name = format!("{}.{}", self.full_name, child_name);
                        return Ok(Self::new(
                            child_name.to_owned(),
                            full_name,
                            entry.get().clone(),
                        ));
                    }
                } else {
                    println!(
                        "ERROR: A dictionary entry at {} has an invalid (non-string) key of {:?}.",
                        self.full_name,
                        entry.key()
                    );
                }
            }
        } else {
            return Err(format!(
                "ERROR: {} is not a dictionary (could not find a child named {}.)",
                self.full_name, child_name
            ));
        }
        result.ok_or(format!(
            "ERROR: {} is missing an entry named {}.",
            self.full_name, child_name
        ))
    }

    pub fn map_entries(&mut self) -> Result<impl Iterator<Item = (String, YamlNode)>, String> {
        if let Yaml::Null = &self.data {
            Ok(Vec::new().into_iter())
        } else if let Yaml::Hash(map) = &mut self.data {
            let mut items = Vec::new();
            for entry in map.entries() {
                if let Yaml::String(child_name) = entry.key() {
                    let full_name = format!("{}.{}", self.full_name, child_name);
                    items.push((
                        child_name.clone(),
                        YamlNode::new(child_name.clone(), full_name, entry.get().clone()),
                    ));
                } else {
                    println!(
                        "ERROR: A dictionary entry at {} has an invalid (non-string) key of {:?}.",
                        self.full_name,
                        entry.key()
                    );
                }
            }
            Ok(items.into_iter())
        } else {
            Err(format!(
                "ERROR: {} is not a valid dictionary.",
                self.full_name,
            ))
        }
    }

    pub fn list_entries(&mut self) -> Result<impl Iterator<Item = YamlNode>, String> {
        if let Yaml::Null = &self.data {
            Ok(Vec::new().into_iter())
        } else if let Yaml::Array(array) = &mut self.data {
            let mut items = Vec::new();
            for (index, entry) in array.iter().enumerate() {
                let full_name = format!("{}[{}]", self.full_name, index);
                items.push(YamlNode::new(
                    format!("{}", index),
                    full_name,
                    entry.clone(),
                ));
            }
            Ok(items.into_iter())
        } else {
            Err(format!(
                "ERROR: {} is not a valid list.",
                self.full_name,
            ))
        }
    }

    pub fn value(&self) -> Result<&str, String> {
        if let Yaml::String(value) = &self.data {
            Ok(&(*value)[..])
        } else {
            Err(format!(
                "ERROR: Expected data at {}, got nothing instead.",
                &self.full_name
            ))
        }
    }

    /// Returns `Ok` if `self.value` can be parsed as a `T`. This is equivalent to doing
    /// `self.value.parse()`. If the value cannot be parsed, a human readable error is
    /// returned along the lines of "ERROR: The value of {path.to.node} is not a valid {type}".
    pub fn parse<T: FromStr>(&self) -> Result<T, String>
    where
        <T as FromStr>::Err: Display,
    {
        if let Yaml::String(value) = &self.data {
            value.parse().map_err(|og_err| {
                format!(
                    "ERROR: The value of {} (\"{}\") is not a valid {}, caused by:\nERROR: {}",
                    self.full_name,
                    value,
                    std::any::type_name::<T>(),
                    og_err
                )
            })
        } else {
            Err(format!(
                "ERROR: Expected a {} at {}, got nothing instead.",
                std::any::type_name::<T>(),
                &self.full_name
            ))
        }
    }

    /// Like `parse()` but uses a custom parse function instead of `str::parse`. Any error returned
    /// by the custom parser will be wrapped with another error specifying the path of this node,
    /// so you do not have to add that information yourself.
    pub fn parse_custom<T>(
        &self,
        parser: impl FnOnce(&str) -> Result<T, String>,
    ) -> Result<T, String> {
        if let Yaml::String(value) = &self.data {
            parser(value).map_err(|err| {
                format!(
                    "ERROR: The value of {} is not valid, caused by:\n{}",
                    self.full_name, err
                )
            })
        } else {
            Err(format!(
                "ERROR: Expected a {} at {}, got nothing instead.",
                std::any::type_name::<T>(),
                &self.full_name
            ))
        }
    }

    /// Like `parse()` but returns an error if the value is outside the specified range. The min/max
    /// bounds are inclusive. You can pass None for a bound if the value should be unbounded in that
    /// direction.
    pub fn parse_ranged<T: FromStr + PartialOrd + Display>(
        &self,
        min: Option<T>,
        max: Option<T>,
    ) -> Result<T, String>
    where
        <T as FromStr>::Err: Display,
    {
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

    /// Returns the index of the item in `possible_values` which matches `self.value`. If
    /// there is no matching item, a human readable error is returned containing the location of
    /// the errror and a list of legal values. E.G.: "ERROR: The value of path.to.node is not one
    /// of 'foo', 'bar', 'baz',". I'm too lazy to get rid of the last comma.
    pub fn parse_enumerated(&self, possible_values: &[&str]) -> Result<usize, String> {
        self.parse_custom(|value| {
            for (index, candidate) in possible_values.iter().enumerate() {
                if value == *candidate {
                    return Ok(index);
                }
            }
            let mut err = format!("ERROR: The value of {} is not one of", self.full_name);
            for value in possible_values {
                err.push_str(&format!(" '{}',", value));
            }
            Err(err)
        })
    }
}

pub fn parse_yaml(input: &str, filename: &str) -> Result<YamlNode, String> {
    let raw = YamlLoader::load_from_str(input);
    let raw = raw.map_err(|err| {
        format!(
            "ERROR: Invalid yaml in file \"{}\", caused by:\nERROR: {}",
            filename, err
        )
    })?;
    if raw.len() != 1 {
        Err(format!(
            "ERROR: While parsing {}, expected 1 document but got {} instead.",
            filename,
            raw.len()
        ))
    } else {
        Ok(YamlNode::new(
            filename.to_owned(),
            filename.to_owned(),
            raw.into_iter().next().unwrap(),
        ))
    }
}
