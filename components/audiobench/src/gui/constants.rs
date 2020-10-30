use std::time::Duration;

/// Pixels of padding between grid squares.
const GRID_P_INT: i32 = 6;
pub const GRID_P: f32 = GRID_P_INT as f32;
/// Number of pixels a single grid square takes.
const GRID_1: i32 = 18;

/// Computes the amount of space (in pixels) taken by the given number of grid tiles, with
/// padding between each tile.
pub const fn grid(num_spaces: i32) -> f32 {
    (GRID_1 * num_spaces + GRID_P_INT * (num_spaces - 1)) as f32
}

/// Like grid, but returns the amount of space used including extra outside padding.
pub const fn fatgrid(num_spaces: i32) -> f32 {
    (GRID_1 * num_spaces + GRID_P_INT * (num_spaces + 1)) as f32
}

/// Computes the coordinate where the provided grid cell begins. For example, 0 would return
/// GRID_P and 1 would return GRID_1 + GRID_P * 2.
pub const fn coord(index: i32) -> f32 {
    (GRID_1 * index + GRID_P_INT * (index + 1)) as f32
}

/// How wide the root window is.
pub const ROOT_WIDTH: f32 = 640.0;
/// How tall the root window is.
pub const ROOT_HEIGHT: f32 = 480.0;
/// How tall the tooltip box should be.
pub const TOOLTIP_HEIGHT: f32 = BIG_FONT_SIZE + GRID_P * 2.0;
/// How tall the bars at the top of the screen are.
pub const HEADER_HEIGHT: f32 = GRID_P + TOOLTIP_HEIGHT + GRID_P + grid(1);
/// How wide tab windows are.
pub const TAB_BODY_WIDTH: f32 = ROOT_WIDTH;
/// How tall tab windows are.
pub const TAB_BODY_HEIGHT: f32 = ROOT_HEIGHT - HEADER_HEIGHT;

pub const KNOB_OUTSIDE_SPACE: f32 = 1.0;
pub const KNOB_INSIDE_SPACE: f32 = 6.0;
pub const KNOB_AUTOMATION_SPACE: f32 = grid(2) / 2.0 - KNOB_OUTSIDE_SPACE - KNOB_INSIDE_SPACE;
pub const KNOB_LANE_GAP: f32 = 1.0;
pub const KNOB_MAX_LANE_SIZE: f32 = 4.0;

pub const KNOB_MENU_LANE_SIZE: f32 = 16.0;
pub const KNOB_MENU_KNOB_OR: f32 = 60.0;
pub const KNOB_MENU_KNOB_IR: f32 = 40.0;
pub const KNOB_MENU_LANE_GAP: f32 = 2.0;

pub const CORNER_SIZE: f32 = 4.0;
pub const FONT_SIZE: f32 = 12.0;
pub const BIG_FONT_SIZE: f32 = 14.0;
pub const TITLE_FONT_SIZE: f32 = grid(1);

pub const MODULE_SHADOW_RADIUS: f32 = 25.0;
pub const POPUP_SHADOW_RADIUS: f32 = 40.0;
pub const JACK_SIZE: f32 = GRID_1 as f32;
pub const JACK_ICON_PADDING: f32 = 1.0;
pub const JACK_SMALL_ICON_SIZE: f32 = 12.0;
// Width of the area dedicated to input or output on each module.
pub const MODULE_IO_WIDTH: f32 = JACK_SIZE + GRID_P as f32;

// Originally 22 but that made grid modules with a reasonable amount of space between them have
// the weird loop-around fallback wire.
pub const WIRE_MIN_SEGMENT_LENGTH: f32 = 21.0;
// Amount of x and y offset required to create a 45 degree line MIN_WIRE_SEGMENT_LENGTH long
pub const WIRE_MIN_DIAGONAL_SIZE: f32 = WIRE_MIN_SEGMENT_LENGTH * std::f32::consts::SQRT_2 / 2.0;
pub const WIRE_SPACING: f32 = (GRID_1 + GRID_P_INT) as f32 / 4.0;

/// How many pixels the mouse must be dragged across to move a knob or slider from its minimum value
/// to its maximum value.
pub const RANGE_DRAG_PIXELS: f32 = 200.0;
/// How many pixels the mouse must be dragged across to double or halve a logarithmic value.
pub const LOG_OCTAVE_PIXELS: f32 = 40.0;
/// How many pixels the mouse must be dragged across to increase or decrease a discrete integer by
/// one step.
pub const DISCRETE_STEP_PIXELS: f32 = 12.0;
/// How many pixels the mouse must move across while being held down for dragging to start.
pub const DRAG_DEADZONE: f32 = 4.0;
/// The maximum amount of time between two clicks for the event to be considered a double-click.
pub const DOUBLE_CLICK_TIME: Duration = Duration::from_millis(500);
/// When holding SnappingModifier, how many steps should be snapped to.
pub const SNAP_STEPS: f32 = 12.0;
/// When holding SnappingModifier and PrecisionModifier, how many steps should be snapped to.
pub const PRECISE_SNAP_STEPS: f32 = 72.0;
/// What to multiply input values by when holding PrecisionModifier.
pub const PRECISION_MULTIPLIER: f32 = 0.2;

const fn hex_color(hex: u32) -> (u8, u8, u8) {
    (
        ((hex >> 16) & 0xFF) as u8,
        ((hex >> 8) & 0xFF) as u8,
        ((hex >> 0) & 0xFF) as u8,
    )
}

// pub const COLOR_DEBUG: (u8, u8, u8) = hex_color(0xFF00FF);
/// The darkest background color.
pub const COLOR_BG0: (u8, u8, u8) = hex_color(0x121520);
/// A medium-intensity background color.
pub const COLOR_BG1: (u8, u8, u8) = hex_color(0x2F434F);
/// The brightest background color.
pub const COLOR_BG2: (u8, u8, u8) = hex_color(0x48525F);
pub const COLOR_ERROR: (u8, u8, u8) = hex_color(0xFF0022);
pub const COLOR_SUCCESS: (u8, u8, u8) = hex_color(0x038c23);
pub const COLOR_WARNING: (u8, u8, u8) = hex_color(0xCC7722);
/// Used to indicate an editable control. COLOR_FG1 should be used for editable values that are
/// represented as text since it is white and COLOR_EDITABLE is red, which is less readable.
pub const COLOR_EDITABLE: (u8, u8, u8) = hex_color(0xFF0022);
pub const COLOR_AUTOMATION: (u8, u8, u8) = hex_color(0xC7D5E8);
// pub const COLOR_AUTOMATION_FOCUSED: (u8, u8, u8) = hex_color(0x54bdff);
/// The darkest foreground color.
pub const COLOR_FG0: (u8, u8, u8) = (0x77, 0x77, 0x77);
/// The brightest foreground color. Use for text and for diagrams.
pub const COLOR_FG1: (u8, u8, u8) = (0xFF, 0xFF, 0xFF);
