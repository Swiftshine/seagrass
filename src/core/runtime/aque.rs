use crate::core::lang;
use crate::core::runtime::Runtime;
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
pub extern "C" fn sg_get_serialized_bytes(
    runtime: *mut Runtime,
    target_name: *const c_char,
) -> *const u8 {
    assert!(!runtime.is_null());

    let target = unsafe {
        (*runtime).get_serialization_target(&CStr::from_ptr(target_name).to_str().unwrap())
    };

    target.as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn sg_get_serialized_size(
    runtime: *mut Runtime,
    target_name: *const c_char,
) -> usize {
    assert!(!runtime.is_null());

    let target = unsafe {
        (*runtime).get_serialization_target(&CStr::from_ptr(target_name).to_str().unwrap())
    };

    target.len()
}

impl Runtime {
    pub fn get_serialization_target(&mut self, target_name: &str) -> &mut Vec<u8> {
        let target = self
            .serialization_targets
            .entry(target_name.to_string())
            .or_default();
        target
    }
}
