#[repr(C)]
pub struct TestStruct {
    att_factor: f32,
}

impl TestStruct {
    fn new() -> Self {
        println!("Created!");
        Self { att_factor: 0.1 }
    }

    fn attenuate(&self, value: f32) -> f32 {
        value * self.att_factor
    }
}

impl Drop for TestStruct {
    fn drop(&mut self) {
        println!("Destroyed!")
    }
}

#[no_mangle]
pub unsafe extern "C" fn ABCreateTestStruct() -> *mut TestStruct {
    Box::into_raw(Box::new(TestStruct::new()))
}

#[no_mangle]
pub unsafe extern "C" fn ABDestroyTestStruct(ts: *mut TestStruct) {
    let data = Box::from_raw(ts);
    drop(data);
}

#[no_mangle]
pub unsafe extern "C" fn ABAttenuate(ts: *mut TestStruct, input: f32) -> f32 {
    (*ts).attenuate(input)
}
