use crate::{
    engine::{
        controls::{self, AnyControl},
        parts as ep,
    },
    gui::module_widgets::WidgetOutline,
    registry::yaml::YamlNode,
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct UserModuleTemplate {
    pub lib_name: String,
    pub module_name: String,
    pub save_id: usize,

    pub label: String,
    pub category: String,
    pub tooltip: String,
    pub size: (i32, i32),
    pub widget_outlines: Vec<WidgetOutline>,

    /// First field is code name, second field is control.
    pub default_controls: Vec<(String, AnyControl)>,
}
