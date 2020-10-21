use crate::engine::parts as ep;
use crate::gui;
use crate::gui::action::{GuiRequest, InstanceRequest, MouseAction};
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::ui_widgets::UiTab;
use crate::registry::save_data::Patch;
use crate::registry::Registry;
use crate::scui_config::Renderer;
use enumflags2::BitFlags;
use scui::{Pos2D, WidgetImpl};
use shared_util::prelude::*;
use std::time::{Duration, Instant};

#[derive(BitFlags, Copy, Clone)]
#[repr(u8)]
pub enum InteractionHint {
    LeftClick = 0x1,
    RightClick = 0x2,
    Scroll = 0x40,
    LeftClickAndDrag = 0x4,
    DoubleClick = 0x8,
    PrecisionModifier = 0x10,
    SnappingModifier = 0x20,
}

#[derive(Clone)]
pub struct Tooltip {
    pub text: String,
    pub interaction: BitFlags<InteractionHint>,
}

impl Default for Tooltip {
    fn default() -> Tooltip {
        Tooltip {
            text: "".to_owned(),
            interaction: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct Status {
    pub text: String,
    pub color: (u8, u8, u8),
}

impl Status {
    fn success(text: String) -> Self {
        Self {
            text,
            color: COLOR_SUCCESS,
        }
    }

    fn error(text: String) -> Self {
        Self {
            text,
            color: COLOR_ERROR,
        }
    }
}

pub use scui::MouseMods;

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum GuiScreen {
    LibraryBrowser,
    PatchBrowser,
    NoteGraph,
    ModuleBrowser,
}

impl GuiScreen {
    fn all() -> Vec<GuiScreen> {
        vec![
            Self::LibraryBrowser,
            Self::PatchBrowser,
            Self::NoteGraph,
            Self::ModuleBrowser,
        ]
    }

    pub fn get_icon_name(&self) -> &'static str {
        match self {
            Self::LibraryBrowser => "factory:library",
            Self::PatchBrowser => "factory:patch_browser",
            Self::NoteGraph => "factory:note",
            Self::ModuleBrowser => "factory:add",
        }
    }

    pub fn get_tooltip_text(&self) -> &'static str {
        match self {
            Self::LibraryBrowser => {
                "Info/Library Browser: View current Audiobench version and installed libraries"
            }
            Self::PatchBrowser => "Patch Browser: Save and load patches",
            Self::NoteGraph => "Note Graph: Edit the module graph used to synthesize notes",
            Self::ModuleBrowser => "Module Browser: Add new modules to the current graph",
        }
    }
}

pub struct GuiState {
    size: Pos2D,
    current_screen: GuiScreen,
    pub registry: Rcrc<Registry>,
    pub status: Option<Status>,
    pub tooltip: Tooltip,
}

scui::widget! {
    pub Root
    Children {
        menu_bar: Option<Rc<gui::ui_widgets::MenuBar>>,
    }
}

impl<'r> WidgetImpl<'r, Renderer<'r>> for Root {

}
