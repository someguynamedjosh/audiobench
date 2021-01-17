use audiobench::*;
use shared_util::prelude::*;

type CreateResult = Result<Instance, ErrorDrawer>;

#[no_mangle]
pub unsafe extern "C" fn ABCreateInstance() -> *mut CreateResult {
    let value = Instance::new().map_err(ErrorDrawer::new);
    Box::into_raw(Box::new(value))
}

#[no_mangle]
pub unsafe extern "C" fn ABDestroyInstance(cr: *mut CreateResult) {
    let data = Box::from_raw(cr);
    drop(data);
}

unsafe fn with_ok<T>(ptr: *mut CreateResult, op: impl FnOnce(&mut Instance) -> T) -> Option<T> {
    (*ptr).as_mut().map(op).ok()
}

#[no_mangle]
pub unsafe extern "C" fn ABUiGetNumIcons(cr: *mut CreateResult) -> i32 {
    with_ok(cr, |instance| instance.registry.borrow().get_num_icons()).unwrap_or_default() as i32
}

#[no_mangle]
pub unsafe extern "C" fn ABUiGetIconData(
    cr: *mut CreateResult,
    icon_index: i32,
    data_buffer: *mut *const u8,
    data_length: *mut i32,
) {
    with_ok(cr, |instance| {
        let registry = instance.registry.borrow();
        let svg_data = registry.borrow_icon_data(icon_index as usize);
        (*data_buffer) = svg_data.as_ptr();
        (*data_length) = svg_data.len() as i32;
    });
}

#[no_mangle]
pub unsafe extern "C" fn ABAudioSetGlobalParameters(
    cr: *mut CreateResult,
    buffer_length: i32,
    sample_rate: i32,
) {
    with_ok(cr, |instance| {
        instance
            .audio_engine
            .borrow_mut()
            .set_global_params(buffer_length as usize, sample_rate as usize)
    });
}

#[no_mangle]
pub unsafe extern "C" fn ABUiSerializePatch(
    cr: *mut CreateResult,
    data_out: *mut *mut u8,
    size_out: *mut u32,
) {
    with_ok(cr, |instance| {
        let data = instance
            .ui_engine
            .borrow()
            .serialize_current_patch()
            .into_bytes()
            .into_boxed_slice();
        *size_out = data.len() as u32;
        *data_out = Box::leak(data).as_mut_ptr();
    });
}

#[no_mangle]
pub unsafe extern "C" fn ABUiCleanupSerializedData(data: *mut u8, size: u32) {
    let slice = std::slice::from_raw_parts_mut(data, size as usize);
    let boxed = Box::from_raw(slice);
    drop(boxed);
}

#[no_mangle]
pub unsafe extern "C" fn ABUiDeserializePatch(
    cr: *mut CreateResult,
    data_in: *mut u8,
    size_in: u32,
) {
    with_ok(cr, |instance| {
        let data = std::slice::from_raw_parts(data_in, size_in as usize);
        let data = Vec::from(data);
        instance.ui_deserialize_patch(&data[..]);
    });
}

#[no_mangle]
pub unsafe extern "C" fn ABAudioStartNote(cr: *mut CreateResult, index: i32, velocity: f32) {
    with_ok(cr, |instance| {
        instance
            .audio_engine
            .borrow_mut()
            .start_note(index as usize, velocity)
    });
}

#[no_mangle]
pub unsafe extern "C" fn ABAudioReleaseNote(cr: *mut CreateResult, index: i32) {
    with_ok(cr, |instance| {
        instance
            .audio_engine
            .borrow_mut()
            .release_note(index as usize)
    });
}

#[no_mangle]
pub unsafe extern "C" fn ABAudioPitchWheel(cr: *mut CreateResult, value: f32) {
    with_ok(cr, |instance| {
        instance.audio_engine.borrow_mut().set_pitch_wheel(value)
    });
}

#[no_mangle]
pub unsafe extern "C" fn ABAudioControl(cr: *mut CreateResult, index: i32, value: f32) {
    with_ok(cr, |instance| {
        instance
            .audio_engine
            .borrow_mut()
            .set_control(index as usize, value)
    });
}

#[no_mangle]
pub unsafe extern "C" fn ABAudioBpm(cr: *mut CreateResult, bpm: f32) {
    with_ok(cr, |instance| {
        instance.audio_engine.borrow_mut().set_bpm(bpm)
    });
}

#[no_mangle]
pub unsafe extern "C" fn ABAudioElapsedTime(cr: *mut CreateResult, time: f32) {
    with_ok(cr, |instance| {
        instance.audio_engine.borrow_mut().set_elapsed_time(time)
    });
}

#[no_mangle]
pub unsafe extern "C" fn ABAudioElapsedBeats(cr: *mut CreateResult, beats: f32) {
    with_ok(cr, |instance| {
        instance.audio_engine.borrow_mut().set_elapsed_beats(beats)
    });
}

#[no_mangle]
pub unsafe extern "C" fn ABAudioRenderAudio(cr: *mut CreateResult) -> *const f32 {
    with_ok(cr, |instance| instance.audio_render_audio().as_ptr()).unwrap_or(std::ptr::null())
}

#[no_mangle]
pub unsafe extern "C" fn ABUiSetGraphicsFunctions(
    cr: *mut CreateResult,
    graphics_fns: GraphicsFunctions,
) {
    with_ok(cr, |instance| instance.graphics_fns = Rc::new(graphics_fns));
}

#[no_mangle]
pub unsafe extern "C" fn ABUiCreateUI(cr: *mut CreateResult) {
    with_ok(cr, |instance| instance.ui_create_ui());
}

#[no_mangle]
pub unsafe extern "C" fn ABUiDrawUI(
    cr: *mut CreateResult,
    graphics_data: *mut i8,
    icon_store: *mut i8,
) {
    match &mut *cr {
        Ok(instance) => instance.ui_draw_ui(graphics_data, icon_store),
        Err(error_drawer) => error_drawer.draw(graphics_data, icon_store),
    }
}

#[no_mangle]
pub unsafe extern "C" fn ABUiDestroyUI(cr: *mut CreateResult) {
    with_ok(cr, |instance| {
        instance.ui_destroy_ui();
    });
}

#[no_mangle]
pub unsafe extern "C" fn ABUiMouseDown(
    cr: *mut CreateResult,
    right_click: bool,
    shift: bool,
    precise: bool,
) {
    let mods = scui::MouseMods {
        right_click,
        snap: shift,
        precise,
    };
    with_ok(cr, |instance| {
        instance.ui_with_gui_mut(|gui| {
            gui.on_mouse_down(&mods);
        })
    });
}

#[no_mangle]
pub unsafe extern "C" fn ABUiMouseMove(
    cr: *mut CreateResult,
    x: f32,
    y: f32,
    right_click: bool,
    shift: bool,
    precise: bool,
) {
    // TOTO: I don't think we're in canvas anymore
    let mods = scui::MouseMods {
        right_click,
        snap: shift,
        precise,
    };
    with_ok(cr, |instance| {
        instance.ui_with_gui_mut(|gui| {
            gui.on_mouse_move((x, y).into(), &mods);
        })
    });
}

#[no_mangle]
pub unsafe extern "C" fn ABUiMouseUp(cr: *mut CreateResult) {
    with_ok(cr, |instance| {
        instance.ui_with_gui_mut(|gui| {
            gui.on_mouse_up();
        })
    });
}

#[no_mangle]
pub unsafe extern "C" fn ABUiScroll(cr: *mut CreateResult, delta: f32) {
    with_ok(cr, |instance| {
        instance.ui_with_gui_mut(|gui| {
            gui.on_scroll(delta);
        })
    });
}

#[no_mangle]
pub unsafe extern "C" fn ABUiKeyPress(cr: *mut CreateResult, key: u8) {
    with_ok(cr, |instance| {
        instance.ui_with_gui_mut(|gui| {
            gui.on_key_press(key as char);
        })
    });
}
