use audiobench::*;
use shared_util::prelude::*;

#[no_mangle]
pub unsafe extern "C" fn ABCreateInstance() -> *mut Instance {
    Box::into_raw(Box::new(Instance::new()))
}

#[no_mangle]
pub unsafe extern "C" fn ABDestroyInstance(instance: *mut Instance) {
    let data = Box::from_raw(instance);
    drop(data);
}

#[no_mangle]
pub unsafe extern "C" fn ABGetNumIcons(instance: *mut Instance) -> i32 {
    (*instance).get_num_icons() as i32
}

#[no_mangle]
pub unsafe extern "C" fn ABGetIconData(
    instance: *mut Instance,
    icon_index: i32,
    data_buffer: *mut *const u8,
    data_length: *mut i32,
) {
    let svg_data = (*instance).borrow_icon_data(icon_index as usize);
    (*data_buffer) = svg_data.as_ptr();
    (*data_length) = svg_data.len() as i32;
}

#[no_mangle]
pub unsafe extern "C" fn ABSetGlobalParameters(
    instance: *mut Instance,
    buffer_length: i32,
    sample_rate: i32,
) {
    (*instance).set_global_params(buffer_length as usize, sample_rate as usize)
}

#[no_mangle]
pub unsafe extern "C" fn ABSerializePatch(
    instance: *mut Instance,
    data_out: *mut *mut u8,
    size_out: *mut u32,
) {
    let data = (*instance)
        .serialize_patch()
        .into_bytes()
        .into_boxed_slice();
    *size_out = data.len() as u32;
    *data_out = Box::leak(data).as_mut_ptr();
}

#[no_mangle]
pub unsafe extern "C" fn ABCleanupSerializedData(data: *mut u8, size: u32) {
    let slice = std::slice::from_raw_parts_mut(data, size as usize);
    let boxed = Box::from_raw(slice);
    drop(boxed);
}

#[no_mangle]
pub unsafe extern "C" fn ABDeserializePatch(
    instance: *mut Instance,
    data_in: *mut u8,
    size_in: u32,
) {
    let data = std::slice::from_raw_parts(data_in, size_in as usize);
    let data = Vec::from(data);
    (*instance).deserialize_patch(&data[..]);
}

#[no_mangle]
pub unsafe extern "C" fn ABStartNote(instance: *mut Instance, index: i32, velocity: f32) {
    (*instance).start_note(index as usize, velocity)
}

#[no_mangle]
pub unsafe extern "C" fn ABReleaseNote(instance: *mut Instance, index: i32) {
    (*instance).release_note(index as usize)
}

#[no_mangle]
pub unsafe extern "C" fn ABPitchWheel(instance: *mut Instance, value: f32) {
    (*instance).set_pitch_wheel(value)
}

#[no_mangle]
pub unsafe extern "C" fn ABControl(instance: *mut Instance, index: i32, value: f32) {
    (*instance).set_control(index as usize, value)
}

#[no_mangle]
pub unsafe extern "C" fn ABBpm(instance: *mut Instance, bpm: f32) {
    (*instance).set_bpm(bpm)
}

#[no_mangle]
pub unsafe extern "C" fn ABSongTime(instance: *mut Instance, time: f32) {
    (*instance).set_elapsed_time(time)
}

#[no_mangle]
pub unsafe extern "C" fn ABSongBeats(instance: *mut Instance, beats: f32) {
    (*instance).set_elapsed_beats(beats)
}

#[no_mangle]
pub unsafe extern "C" fn ABRenderAudio(instance: *mut Instance) -> *const f32 {
    if let Some(audio) = (*instance).render_audio() {
        audio.as_ptr()
    } else {
        std::ptr::null()
    }
}

#[no_mangle]
pub unsafe extern "C" fn ABSetGraphicsFunctions(
    instance: *mut Instance,
    graphics_fns: GraphicsFunctions,
) {
    (*instance).graphics_fns = Rc::new(graphics_fns);
}

#[no_mangle]
pub unsafe extern "C" fn ABCreateUI(instance: *mut Instance) {
    (*instance).create_ui();
}

#[no_mangle]
pub unsafe extern "C" fn ABDrawUI(
    instance: *mut Instance,
    graphics_data: *mut i8,
    icon_store: *mut i8,
) {
    (*instance).draw_ui(graphics_data, icon_store);
}

#[no_mangle]
pub unsafe extern "C" fn ABDestroyUI(instance: *mut Instance) {
    (*instance).destroy_ui();
}

#[no_mangle]
pub unsafe extern "C" fn ABUIMouseDown(
    instance: *mut Instance,
    x: f32,
    y: f32,
    right_click: bool,
    shift: bool,
    precise: bool,
) {
    (*instance).mouse_down(x, y, right_click, shift, precise);
}

#[no_mangle]
pub unsafe extern "C" fn ABUIMouseMove(
    instance: *mut Instance,
    x: f32,
    y: f32,
    right_click: bool,
    shift: bool,
    precise: bool,
) {
    // TOTO: I don't think we're in canvas anymore
    // TODO: Make ABI functions accept floats
    (*instance).mouse_move(x, y, right_click, shift, precise);
}

#[no_mangle]
pub unsafe extern "C" fn ABUIMouseUp(instance: *mut Instance) {
    (*instance).mouse_up();
}

#[no_mangle]
pub unsafe extern "C" fn ABUIScroll(instance: *mut Instance, delta: f32) {
    (*instance).scroll(delta);
}

#[no_mangle]
pub unsafe extern "C" fn ABUIKeyPress(instance: *mut Instance, key: u8) {
    (*instance).key_press(key as char);
}
