use crate::engine::parts as ep;
use crate::engine::static_controls as staticons;
use crate::gui::constants::*;
use crate::gui::module_widgets;
use crate::registry::save_data::Patch;
use scones::make_constructor;
use scui::{MouseBehavior, MouseMods, Vec2D};
use shared_util::prelude::*;


#[derive(Clone, Debug)]
pub enum DropTarget {
    None,
    Autocon(Rcrc<ep::Autocon>),
    Input(Rcrc<ep::Module>, usize),
    Output(Rcrc<ep::Module>, usize),
}
