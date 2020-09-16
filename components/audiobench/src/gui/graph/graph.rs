use super::Module;
use crate::engine::parts as ep;
use crate::gui::action::{DropTarget, GuiAction, MouseAction};
use crate::gui::constants::*;
use crate::gui::graphics::GrahpicsWrapper;
use crate::gui::module_widgets::PopupMenu;
use crate::gui::{Gui, InteractionHint, MouseMods, Tooltip};
use crate::registry::Registry;
use crate::util::*;

pub struct ModuleGraph {
    pub pos: (f32, f32),
    size: (f32, f32),
    offset: Rcrc<(f32, f32)>,
    zoom: Rcrc<f32>,
    graph: Rcrc<ep::ModuleGraph>,
    modules: Vec<Module>,
    // Box because eventually this is going to be dyn.
    detail_menu_widget: Option<Box<dyn PopupMenu>>,
}

impl ModuleGraph {
    pub fn create(registry: &Registry, graph: Rcrc<ep::ModuleGraph>, size: (f32, f32)) -> Self {
        let modules = graph
            .borrow()
            .borrow_modules()
            .iter()
            .map(|module_rc| Module::create(registry, Rc::clone(module_rc)))
            .collect();
        let mut res = Self {
            pos: (0.0, 0.0),
            size,
            zoom: rcrc(1.0),
            offset: rcrc((0.0, 0.0)),
            graph,
            modules,
            detail_menu_widget: None,
        };
        res.recenter();
        res
    }

    pub fn rebuild(&mut self, registry: &Registry) {
        self.detail_menu_widget = None;
        let (mut x1, mut y1, mut x2, mut y2) =
            (std::f32::MAX, std::f32::MAX, std::f32::MIN, std::f32::MIN);
        self.modules.clear();
        for module_rc in self.graph.borrow().borrow_modules() {
            let module_widget = Module::create(registry, Rc::clone(module_rc));
            let pos = module_widget.get_pos();
            if pos.0 < x1 {
                x1 = pos.0;
            }
            if pos.1 < y1 {
                y1 = pos.0;
            }
            let endpos = pos.add(module_widget.size);
            if endpos.0 > x2 {
                x2 = endpos.0;
            }
            if endpos.1 > y2 {
                y2 = endpos.1;
            }
        }
        self.modules = self
            .graph
            .borrow()
            .borrow_modules()
            .iter()
            .map(|module_rc| Module::create(registry, Rc::clone(module_rc)))
            .collect();
        self.recenter();
    }

    fn recenter(&mut self) {
        let (mut x1, mut y1) = (std::f32::MAX, std::f32::MAX);
        let (mut x2, mut y2) = (std::f32::MIN, std::f32::MIN);
        if self.modules.len() == 0 {
            x1 = 0.0;
            y1 = 0.0;
            x2 = 0.0;
            y2 = 0.0;
        }
        for module in &self.modules {
            let corner1 = module.get_pos();
            let corner2 = corner1.add(module.size);
            x1 = x1.min(corner1.0);
            y1 = y1.min(corner1.1);
            x2 = x2.max(corner2.0);
            y2 = y2.max(corner2.1);
        }
        let center = ((x2 - x1) / 2.0 + x1, (y2 - y1) / 2.0 + y1);
        let zoom = *self.zoom.borrow();
        let offset = center.sub((self.size.0 / zoom / 2.0, self.size.1 / zoom / 2.0));
        *self.offset.borrow_mut() = (-offset.0 as f32, -offset.1 as f32);
    }

    pub fn add_module(&mut self, registry: &Registry, mut module: ep::Module) {
        module.pos = *self.offset.borrow();
        module.pos = (-module.pos.0, -module.pos.1);
        let module = rcrc(module);
        self.graph.borrow_mut().add_module(Rc::clone(&module));
        self.modules.push(Module::create(registry, module));
    }

    pub fn remove_module(&mut self, module: &Rcrc<ep::Module>) {
        self.graph.borrow_mut().remove_module(module);
        let index = self
            .modules
            .iter()
            .position(|e| std::ptr::eq(e.module.as_ref(), module.as_ref()))
            .unwrap();
        self.modules.remove(index);
    }

    fn translate_mouse_pos(&self, mouse_pos: (f32, f32)) -> (f32, f32) {
        let offset = self.offset.borrow();
        let zoom = *self.zoom.borrow();
        (
            ((mouse_pos.0) as f32 / zoom - offset.0) as f32 - self.pos.0,
            ((mouse_pos.1) as f32 / zoom - offset.1) as f32 - self.pos.1,
        )
    }

    pub fn respond_to_mouse_press(
        &mut self,
        mouse_pos: (f32, f32),
        mods: &MouseMods,
    ) -> MouseAction {
        let mouse_pos = self.translate_mouse_pos(mouse_pos);
        if let Some(widget) = &self.detail_menu_widget {
            let local_pos = mouse_pos.sub(widget.get_pos());
            if local_pos.inside(widget.get_bounds()) {
                return widget
                    .respond_to_mouse_press(local_pos, mods)
                    .scaled(Rc::clone(&self.zoom));
            } else {
                self.detail_menu_widget = None;
            }
        }
        for module in self.modules.iter().rev() {
            let action = module.respond_to_mouse_press(mouse_pos, mods);
            if !action.is_none() {
                return action.scaled(Rc::clone(&self.zoom));
            }
        }
        MouseAction::PanOffset(Rc::clone(&self.offset)).scaled(Rc::clone(&self.zoom))
    }

    pub fn on_scroll(&mut self, delta: f32) -> Option<GuiAction> {
        let mut offset = self.offset.borrow_mut();
        let mut zoom = self.zoom.borrow_mut();
        *offset = offset.sub((self.size.0 / *zoom / 2.0, self.size.1 / *zoom / 2.0));
        *zoom *= f32::exp(delta / 3.0);
        *offset = offset.add((self.size.0 / *zoom / 2.0, self.size.1 / *zoom / 2.0));
        None
    }

    pub fn get_drop_target_at(&self, mouse_pos: (f32, f32)) -> DropTarget {
        let mouse_pos = self.translate_mouse_pos(mouse_pos);
        for module in &self.modules {
            let target = module.get_drop_target_at(mouse_pos);
            if !target.is_none() {
                return target;
            }
        }
        DropTarget::None
    }

    pub fn get_tooltip_at(&self, mouse_pos: (f32, f32)) -> Option<Tooltip> {
        let mouse_pos = self.translate_mouse_pos(mouse_pos);
        if let Some(dmw) = &self.detail_menu_widget {
            let local_pos = mouse_pos.sub(dmw.get_pos());
            if local_pos.inside(dmw.get_bounds()) {
                return dmw.get_tooltip_at(local_pos);
            }
        }
        for module in &self.modules {
            if let Some(tooltip) = module.get_tooltip_at(mouse_pos) {
                return Some(tooltip);
            }
        }
        Some(Tooltip {
            text: "".to_owned(),
            interaction: InteractionHint::Scroll.into(),
        })
    }

    pub fn draw(&self, g: &mut GrahpicsWrapper, gui_state: &Gui) {
        let offset = self.offset.borrow();
        let offset = (offset.0 as f32, offset.1 as f32);
        g.push_state();
        g.apply_scale(*self.zoom.borrow());
        g.apply_offset(offset.0 + self.pos.0, offset.1 + self.pos.1);
        let (mx, my) = self.translate_mouse_pos(gui_state.get_current_mouse_pos());
        let highlight = if gui_state.is_dragging() {
            let cma = gui_state.borrow_current_mouse_action();
            if let MouseAction::ConnectInput(module, index) = cma {
                let typ = module.borrow().template.borrow().inputs[*index].get_type();
                // Highlight outputs with typ.
                Some((true, typ))
            } else if let MouseAction::ConnectOutput(module, index) = cma {
                let typ = module.borrow().template.borrow().outputs[*index].get_type();
                // Highlight inputs with typ.
                Some((false, typ))
            } else {
                None
            }
        } else {
            None
        };
        for layer in 0..4 {
            for module in &self.modules {
                module.draw(g, (mx, my), highlight, layer);
            }
        }
        if let Some(widget) = &self.detail_menu_widget {
            widget.draw(g);
        }
        if gui_state.is_dragging() {
            let cma = gui_state.borrow_current_mouse_action();
            if let MouseAction::ConnectInput(module, index) = cma {
                let module_ref = module.borrow();
                let (sx, sy) = Module::input_position(&*module_ref, *index as i32);
                g.set_color(&COLOR_FG1);
                g.stroke_line(sx, sy, mx, my, 2.0);
            } else if let MouseAction::ConnectOutput(module, index) = cma {
                let module_ref = module.borrow();
                let (sx, sy) = Module::output_position(&*module_ref, *index as i32);
                g.set_color(&COLOR_FG1);
                g.stroke_line(sx, sy, mx, my, 2.0);
            }
        }
        g.pop_state();
    }

    pub fn open_menu(&mut self, menu: Box<dyn PopupMenu>) {
        self.detail_menu_widget = Some(menu);
    }
}
