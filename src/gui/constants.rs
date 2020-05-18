/// Pixels of padding between grid squares.
pub const GRID_P: i32 = 8;
/// Number of pixels a single grid square takes.
const GRID_1: i32 = 20;

/// Computes the amount of space (in pixels) taken by the given number of grid tiles, with
/// padding between each tile.
pub const fn grid(num_spaces: i32) -> i32 {
    GRID_1 * num_spaces + GRID_P * (num_spaces - 1)
}

/// Like grid, but returns the amount of space used including extra outside padding. Use  
/// alongside the fatcoord function.
pub const fn fatgrid(num_spaces: i32) -> i32 {
    GRID_1 * num_spaces + GRID_P * (num_spaces + 1)
}

/// Computes the coordinate where the provided grid cell begins. For example, 0 would return
/// GRID_P and 1 would return GRID_1 + GRID_P * 2.
pub const fn coord(index: i32) -> i32 {
    GRID_1 * index + GRID_P * (index + 1)
}
/// Like coord, but allows space for extra padding. Use alongside the fatgrid function.
pub const fn fatcoord(index: i32) -> i32 {
    GRID_1 * index + GRID_P * index
}

pub const KNOB_OUTSIDE_SPACE: i32 = 1;
pub const KNOB_INSIDE_SPACE: i32 = 6;
pub const KNOB_AUTOMATION_SPACE: i32 = grid(2) / 2 - KNOB_OUTSIDE_SPACE - KNOB_INSIDE_SPACE;
pub const KNOB_LANE_GAP: i32 = 1;
pub const KNOB_MAX_LANE_SIZE: i32 = 4;

pub const KNOB_MENU_LANE_SIZE: i32 = 16;
pub const KNOB_MENU_KNOB_OR: i32 = 60;
pub const KNOB_MENU_KNOB_IR: i32 = 40;
pub const KNOB_MENU_LANE_GAP: i32 = 2;

pub const CORNER_SIZE: i32 = 4;
pub const JACK_SIZE: i32 = GRID_1;
pub const JACK_DEFAULT_SIZE: i32 = JACK_SIZE - 4;
pub const JACK_ICON_PADDING: i32 = 1;
pub const JACK_SMALL_ICON_SIZE: i32 = 12;
// Width of the area dedicated to input or output on each module.
pub const MODULE_IO_WIDTH: i32 = JACK_SIZE + GRID_P;

pub const WIRE_MIN_SEGMENT_LENGTH: i32 = 22;
// Amount of x and y offset required to create a 45 degree line MIN_WIRE_SEGMENT_LENGTH long
pub const WIRE_MIN_DIAGONAL_SIZE: i32 =
    (WIRE_MIN_SEGMENT_LENGTH as f32 * std::f32::consts::SQRT_2 / 2.0) as i32;
pub const WIRE_SPACING: i32 = GRID_P;

const fn hex_color(hex: u32) -> (u8, u8, u8) {
    (
        ((hex >> 16) & 0xFF) as u8,
        ((hex >> 8) & 0xFF) as u8,
        ((hex >> 0) & 0xFF) as u8,
    )
}

pub const COLOR_DEBUG: (u8, u8, u8) = hex_color(0xFF00FF);
pub const COLOR_BG: (u8, u8, u8) = hex_color(0x121520);
pub const COLOR_SURFACE: (u8, u8, u8) = hex_color(0x48525F);
pub const COLOR_IO_AREA: (u8, u8, u8) = hex_color(0x2F434F);
pub const COLOR_KNOB: (u8, u8, u8) = hex_color(0xFF0022);
pub const COLOR_AUTOMATION: (u8, u8, u8) = hex_color(0xC7D5E8);
pub const COLOR_AUTOMATION_FOCUSED: (u8, u8, u8) = hex_color(0x54bdff);
pub const COLOR_TEXT: (u8, u8, u8) = (0xFF, 0xFF, 0xFF);
pub const COLOR_MUTED_TEXT: (u8, u8, u8) = (0x77, 0x77, 0x77);
