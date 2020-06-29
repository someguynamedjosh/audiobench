use crate::gui::constants::*;
use crate::gui::graphics::GrahpicsWrapper;

// This code is not intended to be maintainable. It was created by madly scribbling on graph paper
// for five hours. If it is broken the only way to fix it is to start over.
pub(super) fn draw_automation_wire(
    g: &mut GrahpicsWrapper,
    face_down: bool,
    x1: f32,
    y1: f32,
    x2: f32,
    mut y2: f32,
) {
    const S: f32 = WIRE_MIN_SEGMENT_LENGTH;
    const D: f32 = WIRE_MIN_DIAGONAL_SIZE;
    const SD: f32 = S + D;
    const W: f32 = 2.0;
    let mut stroke_line: Box<dyn FnMut(f32, f32, f32, f32)> = if face_down {
        let pivot = y1;
        y2 = pivot - (y2 - pivot);
        Box::new(move |x1, y1, x2, y2| {
            let y1 = pivot - (y1 - pivot);
            let y2 = pivot - (y2 - pivot);
            g.stroke_line(x1, y1, x2, y2, W);
        })
    } else {
        Box::new(|x1, y1, x2, y2| {
            g.stroke_line(x1, y1, x2, y2, W);
        })
    };
    let dx = x2 - x1;
    let dy = y2 - y1;
    if dx <= -D {
        if dy >= D {
            stroke_line(x1, y1, x1 + S, y1);
            stroke_line(x1 + S, y1, x1 + SD, y1 + D);
            let down_segment_length = dy - D;
            let left_segment_length = -dx - D;
            let diagonal = down_segment_length.min(left_segment_length) / 2.0;
            let dsl = down_segment_length - diagonal + S;
            let lsl = left_segment_length - diagonal + S;
            let diagonal = diagonal + D;
            stroke_line(x1 + SD, y1 + D, x1 + SD, y1 + D + dsl);
            stroke_line(x1 + SD, y1 + D + dsl, x1 + SD - diagonal, y2 + SD);
            stroke_line(x2 + D + lsl, y2 + SD, x2 + D, y2 + SD);
            stroke_line(x2 + D, y2 + SD, x2, y2 + S);
            stroke_line(x2, y2 + S, x2, y2);
        } else if dy <= -SD * 2.0 - D {
            stroke_line(x1, y1, x1 + S, y1);
            stroke_line(x1 + S, y1, x1 + SD, y1 - D);
            stroke_line(x1 + SD, y1 - D, x1 + SD, y1 - SD);
            stroke_line(x1 + SD, y1 - SD, x1 + S, y1 - SD - D);
            let up_segment_length = -SD * 2.0 - D - dy;
            let left_segment_length = -dx - D;
            let diagonal = up_segment_length.min(left_segment_length) / 2.0;
            let usl = up_segment_length - diagonal + S;
            // let lsl = left_segment_length - diagonal + S;
            let diagonal = diagonal + D;
            stroke_line(x1 + S, y1 - SD - D, x2 + diagonal, y1 - SD - D);
            stroke_line(x2 + diagonal, y1 - SD - D, x2, y2 + usl);
            stroke_line(x2, y2 + usl, x2, y2);
        } else {
            stroke_line(x1, y1, x1 + S, y1);
            stroke_line(x1 + S, y1, x1 + SD, y1 + D);
            stroke_line(x1 + SD, y1 + D, x1 + SD, y1 + SD);
            stroke_line(x1 + SD, y1 + SD, x1 + S, y1 + SD + D);
            let left_segment_length = -dx - D;
            let up_segment_length = D - dy;
            let diagonal = left_segment_length.min(up_segment_length) / 2.0;
            let usl = up_segment_length - diagonal + S;
            let lsl = left_segment_length - diagonal + S;
            // let diagonal = diagonal + D;
            stroke_line(x1 + S, y1 + SD + D, x1 + S - lsl, y1 + SD + D);
            stroke_line(x1 + S - lsl, y1 + SD + D, x2, y2 + usl);
            stroke_line(x2, y2 + usl, x2, y2);
        }
    } else if dx >= SD && dy <= -SD {
        let right_segment_length = dx - SD;
        let up_segment_length = -SD - dy;
        let diagonal = right_segment_length.min(up_segment_length) / 2.0;
        let rsl = right_segment_length - diagonal + S;
        // let usl = up_segment_length - diagonal + S;
        let diagonal = diagonal + D;
        stroke_line(x1, y1, x1 + rsl, y1);
        stroke_line(x1 + rsl, y1, x2, y1 - diagonal);
        stroke_line(x2, y1 - diagonal, x2, y2);
    } else if dx >= -D && dy <= -SD * 2.0 - D {
        let right_segment_length = dx + D;
        let up_segment_length = -SD * 2.0 - D - dy;
        let diagonal = right_segment_length.min(up_segment_length) / 2.0;
        let rsl = right_segment_length - diagonal + S;
        let usl = up_segment_length - diagonal + S;
        let diag = diagonal + D;
        stroke_line(x1, y1, x1 + rsl, y1);
        stroke_line(x1 + rsl, y1, x2 + SD + D, y1 - diag);
        stroke_line(x2 + SD + D, y1 - diag, x2 + SD + D, y1 - diag - S);
        stroke_line(x2 + SD + D, y1 - diag - S, x2 + SD, y2 + usl + D);
        stroke_line(x2 + SD, y2 + D + usl, x2 + D, y2 + D + usl);
        stroke_line(x2 + D, y2 + D + usl, x2, y2 + usl);
        stroke_line(x2, y2 + usl, x2, y2);
    } else if dx >= SD * 2.0 + D && dy >= D {
        let right_segment_length = dx - SD * 2.0 - D;
        let down_segment_length = dy - D;
        let diagonal = right_segment_length.min(down_segment_length) / 2.0;
        // let rsl = right_segment_length - diagonal + S;
        let dsl = down_segment_length - diagonal + S;
        let diagonal = diagonal + D;
        stroke_line(x1, y1, x1 + S, y1);
        stroke_line(x1 + S, y1, x1 + SD, y1 + D);
        stroke_line(x1 + SD, y1 + D, x1 + SD, y1 + D + dsl);
        stroke_line(x1 + SD, y1 + D + dsl, x1 + SD + diagonal, y2 + SD);
        stroke_line(x1 + SD + diagonal, y2 + SD, x2 - D, y2 + SD);
        stroke_line(x2 - D, y2 + SD, x2, y2 + S);
        stroke_line(x2, y2 + S, x2, y2);
    } else if dx >= SD * 2.0 + D {
        let right_segment_length = dx - SD * 2.0 - D;
        let up_segment_length = -dy + D;
        let diagonal = right_segment_length.min(up_segment_length) / 2.0;
        // let rsl = right_segment_length - diagonal + S;
        let usl = up_segment_length - diagonal + S;
        let diagonal = diagonal + D;
        stroke_line(x1, y1, x1 + S, y1);
        stroke_line(x1 + S, y1, x1 + SD, y1 + D);
        stroke_line(x1 + SD, y1 + D, x1 + SD, y1 + SD);
        stroke_line(x1 + SD, y1 + SD, x1 + SD + D, y1 + SD + D);
        stroke_line(x1 + SD + D, y1 + SD + D, x2 - diagonal, y1 + SD + D);
        stroke_line(x2 - diagonal, y1 + SD + D, x2, y2 + usl);
        stroke_line(x2, y2 + usl, x2, y2);
    } else if dy >= D {
        let down_segment_length = dy - D;
        let right_segment_length = dx + D;
        let diagonal = down_segment_length.min(right_segment_length) / 2.0;
        // let dsl = down_segment_length - diagonal + S;
        let rsl = right_segment_length - diagonal + S;
        let diagonal = diagonal + D;
        stroke_line(x1, y1, x1 + rsl, y1);
        stroke_line(x1 + rsl, y1, x1 + rsl + diagonal, y1 + diagonal);
        stroke_line(x2 + SD + D, y1 + diagonal, x2 + SD + D, y2 + S);
        stroke_line(x2 + SD + D, y2 + S, x2 + SD, y2 + SD);
        stroke_line(x2 + SD, y2 + SD, x2 + D, y2 + SD);
        stroke_line(x2 + D, y2 + SD, x2, y2 + S);
        stroke_line(x2, y2 + S, x2, y2);
    } else {
        let usl = D - dy + S;
        let rsl = dx + D + S;
        stroke_line(x1, y1, x1 + rsl, y1);
        stroke_line(x1 + rsl, y1, x1 + rsl + D, y1 + D);
        stroke_line(x1 + rsl + D, y1 + D, x1 + rsl + D, y1 + SD);
        stroke_line(x1 + rsl + D, y1 + SD, x1 + rsl, y1 + SD + D);
        stroke_line(x1 + rsl, y1 + SD + D, x2 + D, y1 + SD + D);
        stroke_line(x2 + D, y1 + SD + D, x2, y2 + usl);
        stroke_line(x2, y2 + usl, x2, y2);
    }
}

// x1, y1 is coord of input, x2, y2 is coord of output.
pub(super) fn draw_io_wire(g: &mut GrahpicsWrapper, x1: f32, y1: f32, x2: f32, mut y2: f32) {
    const S: f32 = WIRE_MIN_SEGMENT_LENGTH;
    const D: f32 = WIRE_MIN_DIAGONAL_SIZE;
    const SD: f32 = S + D;
    const W: f32 = 2.0;
    let mut stroke_line: Box<dyn FnMut(f32, f32, f32, f32)> = if y2 < y1 {
        let pivot = y1;
        // Since pivot is y1, y1 remains unchanged.
        y2 = pivot - (y2 - pivot);
        Box::new(move |x1, y1, x2, y2| {
            let y1 = pivot - (y1 - pivot);
            let y2 = pivot - (y2 - pivot);
            g.stroke_line(x1, y1, x2, y2, W);
        })
    } else {
        Box::new(|x1, y1, x2, y2| {
            g.stroke_line(x1, y1, x2, y2, W);
        })
    };
    let dx = x2 - x1;
    let dy = y2 - y1;
    if -dx - S * 2.0 >= dy {
        let diagonal = dy;
        let lsl = (-dx - diagonal) / 2.0;
        stroke_line(x1, y1, x1 - lsl, y1);
        stroke_line(x1 - lsl, y1, x2 + lsl, y2);
        stroke_line(x2 + lsl, y2, x2, y2);
    } else if dx <= -SD * 2.0 && dy >= SD + D {
        let left_segment_length = -dx - SD * 2.0;
        let up_segment_length = dy - SD - D;
        let diag = left_segment_length.min(up_segment_length) / 2.0;
        let lsl = left_segment_length - diag + S;
        // let usl = up_segment_length - diag + S;
        let diag = diag + D;
        stroke_line(x1, y1, x1 - S, y1);
        stroke_line(x1 - S, y1, x1 - SD, y1 + D);
        stroke_line(x1 - SD, y1 + D, x1 - SD, y2 - diag);
        stroke_line(x1 - SD, y2 - diag, x2 + lsl, y2);
        stroke_line(x2 + lsl, y2, x2, y2);
    } else if dx >= -S && dy >= 2.0 * S + 4.0 * D {
        let right_segment_length = dx + S;
        let up_segment_length = dy - 2.0 * S - 4.0 * D;
        let diag = right_segment_length.min(up_segment_length) / 2.0;
        let rsl = right_segment_length - diag + S;
        let usl = up_segment_length - diag + S;
        // let diag = diag + D;
        stroke_line(x1, y1, x1 - S, y1);
        stroke_line(x1 - S, y1, x1 - SD, y1 + D);
        stroke_line(x1 - SD, y1 + D, x1 - SD, y1 + SD);
        stroke_line(x1 - SD, y1 + SD, x1 - S, y1 + SD + D);
        stroke_line(x1 - S, y1 + SD + D, x1 - S + rsl, y1 + SD + D);
        stroke_line(x1 - S + rsl, y1 + SD + D, x2 + SD, y2 - D - usl);
        stroke_line(x2 + SD, y2 - D - usl, x2 + SD, y2 - D);
        stroke_line(x2 + SD, y2 - D, x2 + S, y2);
        stroke_line(x2 + S, y2, x2, y2);
    } else if dy >= 2.0 * S + 4.0 * D {
        let left_segment_length = -dx - S;
        let up_segment_length = dy - 2.0 * S - 4.0 * D;
        let diag = left_segment_length.min(up_segment_length) / 2.0;
        let lsl = left_segment_length - diag + S;
        let usl = up_segment_length - diag + S;
        let diag = diag + D;
        stroke_line(x1, y1, x1 - S, y1);
        stroke_line(x1 - S, y1, x1 - SD, y1 + D);
        stroke_line(x1 - SD, y1 + D, x1 - SD, y1 + SD);
        stroke_line(x1 - SD, y1 + SD, x1 - S, y1 + SD + D);
        stroke_line(x1 - S, y1 + SD + D, x1, y1 + SD + D);
        stroke_line(x1, y1 + SD + D, x1 + D, y2 - diag - usl);
        stroke_line(x1 + D, y2 - diag - usl, x1 + D, y2 - diag);
        stroke_line(x1 + D, y2 - diag, x2 + lsl, y2);
        stroke_line(x2 + lsl, y2, x2, y2);
    } else if dx >= -S {
        let right_segment_length = dx + S;
        let up_segment_length = dy;
        let diag = right_segment_length.min(up_segment_length) / 2.0;
        // let rsl = right_segment_length - diag + S;
        let usl = up_segment_length - diag + S;
        let diag = diag + D;
        stroke_line(x1, y1, x1 - S, y1);
        stroke_line(x1 - S, y1, x1 - SD, y1 - D);
        stroke_line(x1 - SD, y1 - D, x1 - SD, y1 - SD);
        stroke_line(x1 - SD, y1 - SD, x1 - D, y1 - SD - D);
        stroke_line(x1 - D, y1 - SD - D, x2 + SD - diag, y1 - SD - D);
        stroke_line(x2 + SD - diag, y1 - SD - D, x2 + SD, y2 - D - usl);
        stroke_line(x2 + SD, y2 - D - usl, x2 + SD, y2 - D);
        stroke_line(x2 + SD, y2 - D, x2 + S, y2);
        stroke_line(x2 + S, y2, x2, y2);
    } else {
        // let lsl = -dx;
        // let usl = dy + S;
        stroke_line(x1, y1, x2, y1);
        stroke_line(x2, y1, x2 - D, y1 - D);
        stroke_line(x2 - D, y1 - D, x2 - D, y1 - SD);
        stroke_line(x2 - D, y1 - SD, x2, y1 - SD - D);
        stroke_line(x2, y1 - SD - D, x2 + S, y1 - SD - D);
        stroke_line(x2 + S, y1 - SD - D, x2 + SD, y1 - SD);
        stroke_line(x2 + SD, y1 - SD, x2 + SD, y2 - D);
        stroke_line(x2 + SD, y2 - D, x2 + S, y2);
        stroke_line(x2 + S, y2, x2, y2);
    }
}

#[derive(Clone)]
pub(in crate::gui) struct WireTracker {
    module_height: f32,
    top_slots: Vec<bool>,
    bottom_slots: Vec<bool>,
    wires: Vec<((f32, f32), (f32, f32), bool)>,
}

impl WireTracker {
    pub(super) fn new(module_size: (f32, f32)) -> Self {
        let num_slots = (module_size.0 - MODULE_IO_WIDTH * 2.0 - JACK_SIZE) / WIRE_SPACING;
        Self {
            module_height: module_size.1,
            top_slots: vec![false; num_slots as usize],
            bottom_slots: vec![false; num_slots as usize],
            wires: Vec::new(),
        }
    }
    pub fn add_wire(&mut self, source_coord: (f32, f32), widget_coord: (f32, f32)) {
        let slot_index = ((widget_coord.0 - MODULE_IO_WIDTH - JACK_SIZE) / WIRE_SPACING) as usize;
        let slot_index = slot_index.min(self.top_slots.len() - 1);
        let top = widget_coord.1 <= self.module_height / 2.0;
        let slots = if top {
            &mut self.top_slots
        } else {
            &mut self.bottom_slots
        };
        let mut left_slot = slot_index;
        let mut right_slot = slot_index;
        let empty_slot;
        loop {
            if !slots[left_slot] {
                empty_slot = left_slot;
                break;
            }
            if !slots[right_slot] {
                empty_slot = right_slot;
                break;
            }
            if left_slot > 0 {
                left_slot -= 1;
            }
            if right_slot < slots.len() - 1 {
                right_slot += 1;
            } else if left_slot == 0 {
                // No empty slot. This prevents an infinite loop.
                empty_slot = slot_index;
                break;
            }
        }
        slots[empty_slot] = true;

        let (endx, endy) = (
            empty_slot as f32 * WIRE_SPACING + WIRE_SPACING / 2.0 + MODULE_IO_WIDTH + JACK_SIZE,
            if top { 0.0 } else { self.module_height },
        );
        self.wires.push((source_coord, (endx, endy), top));
    }

    pub fn draw_wires(self, g: &mut GrahpicsWrapper, target_offset: (f32, f32)) {
        for (source, target, face_down) in self.wires {
            draw_automation_wire(
                g,
                face_down,
                source.0,
                source.1,
                target.0 + target_offset.0,
                target.1 + target_offset.1,
            );
        }
    }
}
