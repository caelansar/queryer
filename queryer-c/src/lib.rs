use libc::c_char;
use std::{
    ffi::{CStr, CString},
    panic::catch_unwind,
    ptr,
};

#[no_mangle]
pub extern "C" fn hello(name: *const c_char) -> *const c_char {
    format!("hello {}!\0", unsafe {
        CStr::from_ptr(name).to_str().unwrap()
    })
    .as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn query(sql: *const c_char, output: *const c_char) -> *const c_char {
    let result = catch_unwind(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        if let Ok(s) = unsafe { CStr::from_ptr(sql).to_str() } {
            let mut data = rt.block_on(async { queryer_rs::query(s).await.unwrap() });
            let output = {
                if output.is_null() {
                    Ok("json")
                } else {
                    unsafe { CStr::from_ptr(output).to_str() }
                }
            };
            match output {
                Ok("csv") => CString::new(data.to_csv().unwrap()).unwrap().into_raw(),
                Ok("json") => CString::new(data.to_json().unwrap()).unwrap().into_raw(),
                Ok(_) => ptr::null(),
                Err(_) => ptr::null(),
            }
        } else {
            ptr::null()
        }
    });
    match result {
        Ok(s) => s,
        Err(_) => ptr::null(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn free_str(s: *mut c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s);
    }
}
