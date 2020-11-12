use crate::engine::parts as ep;
use crate::engine::static_controls as staticons;
use crate::engine::UiThreadEngine;
use crate::gui::constants::*;
use crate::gui::module_widgets;
use crate::registry::save_data::Patch;
use crate::scui_config::GuiState;
use scones::make_constructor;
use scui::{GuiInterfaceProvider, MouseBehavior, MouseMods, Vec2D};
use shared_util::prelude::*;

#[derive(Clone, Debug)]
pub enum DropTarget {
    None,
    Autocon(Rcrc<ep::Autocon>),
    Input(Rcrc<ep::Module>, usize),
    Output(Rcrc<ep::Module>, usize),
}

#[make_constructor(new)]
pub struct MutateStaticon {
    engine: Rcrc<UiThreadEngine>,
    mutator: Box<dyn FnOnce() -> staticons::StaticonUpdateRequest>,
}

impl MutateStaticon {
    pub fn wrap<W, M>(widget: &W, mutator: M) -> Option<Box<dyn MouseBehavior>>
    where
        W: GuiInterfaceProvider<GuiState>,
        M: FnOnce() -> staticons::StaticonUpdateRequest + 'static,
    {
        let int = widget.provide_gui_interface();
        let engine = Rc::clone(&int.state.borrow().engine);
        Some(Box::new(Self::new(engine, Box::new(mutator))))
    }
}

impl MouseBehavior for MutateStaticon {
    fn on_click(self: Box<Self>) {
        let update = (self.mutator)();
        match update {
            staticons::StaticonUpdateRequest::Nothing => (),
            staticons::StaticonUpdateRequest::UpdateDynData => {
                self.engine.borrow_mut().reload_staticon_dyn_data();
            }
            staticons::StaticonUpdateRequest::UpdateCode => {
                self.engine
                    .borrow_mut()
                    .recompile()
                    .expect("Compile should only fail if a feedback loop was introduced.");
            }
        }
    }
}
