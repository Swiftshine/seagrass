use crate::core::lang;
use crate::core::runtime::{Runtime, RuntimeError, RuntimeResult};
use std::ffi::CStr;
use std::fs;
use std::os::raw::c_char;

// C bindings

#[unsafe(no_mangle)]
pub extern "C" fn sg_create_runtime(string: *const c_char) -> *mut Runtime {
    assert!(!string.is_null());

    let path = unsafe {
        let c_str = CStr::from_ptr(string);
        let string = c_str.to_string_lossy().to_string();
        string.into()
    };

    let runtime = Runtime::new(path);
    Box::into_raw(Box::new(runtime))
}

#[unsafe(no_mangle)]
pub extern "C" fn sg_free_runtime(runtime: *mut Runtime) {
    if runtime.is_null() {
        // reclaim
        unsafe {
            let _ = Box::from_raw(runtime);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sg_execute_runtime(runtime: *mut Runtime) {
    assert!(!runtime.is_null());

    unsafe {
        let runtime = &mut *runtime;
        let path = runtime.base_dir();
        let contents = fs::read_to_string(path).unwrap();
        let program = lang::build_program(&contents).unwrap();
        runtime.execute(&program).unwrap();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sg_get_serialized_result(runtime: *mut Runtime) -> *const u8 {
    assert!(!runtime.is_null());

    let target = unsafe { (*runtime).get_serialization_target_mut().unwrap() };

    target.data.as_ptr()
}

#[derive(Debug)]
pub struct SerializationTarget {
    pub data: Vec<u8>,
}

impl SerializationTarget {
    fn new() -> Self {
        Self { data: Vec::new() }
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

impl Runtime {
    pub fn create_serialization_target(&mut self) {
        self.destroy_serialization_target();
        self.serialization_target = Some(SerializationTarget::new())
    }

    pub fn destroy_serialization_target(&mut self) {
        self.serialization_target = None;
    }

    pub fn has_serialization_target(&self) -> bool {
        self.serialization_target.is_some()
    }

    pub fn get_serialization_target_mut(&mut self) -> RuntimeResult<&mut SerializationTarget> {
        self.serialization_target
            .as_mut()
            .ok_or(RuntimeError::NoSerializationTarget)
    }
}
