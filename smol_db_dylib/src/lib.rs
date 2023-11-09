use std::ffi::{c_char, CStr, CString};
use smol_db_client::{DBSuccessResponse, SmolDbClient};

pub const ERROR_STATE: u8 = 1;
pub const DATA_NOT_FOUND_STATE: u8 = 2;
pub const OK_STATE: u8 = 0;

pub struct FFISmolDBClient {
    pub client: SmolDbClient,
}

impl FFISmolDBClient {
    pub fn new(ip: &str) -> FFISmolDBClient {
        Self { client: SmolDbClient::new(ip).unwrap() }
    }
}

#[no_mangle]
pub extern "C" fn smol_db_client_new(ip: *const c_char) -> *mut FFISmolDBClient {
    let ip_address = unsafe {
        assert!(!ip.is_null());
        CStr::from_ptr(ip).to_str().unwrap()
    };

    // TODO: fix this unwrap ?
    Box::into_raw(Box::new(FFISmolDBClient::new(ip_address)))
}

#[no_mangle]
pub extern "C" fn smol_db_client_free(client_ptr: *mut FFISmolDBClient) {
    if client_ptr.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(client_ptr);
    }
}

#[no_mangle]
extern "C" fn smol_db_client_read_db(client_ptr: *mut FFISmolDBClient, name: *const c_char,location: *const c_char) -> *const c_char {
    let client = unsafe {
        assert!(!client_ptr.is_null());
        &mut *client_ptr
    };
    let db_name = unsafe {
        assert!(!name.is_null());
        CStr::from_ptr(name).to_str().unwrap()
    };
    let db_location = unsafe {
        assert!(!location.is_null());
        CStr::from_ptr(location).to_str().unwrap()
    };
    return match client.client.read_db(db_name,db_location) {
        Ok(resp) => {
            match resp {
                DBSuccessResponse::SuccessNoData => {
                    CString::new([DATA_NOT_FOUND_STATE]).unwrap().into_raw()
                }
                DBSuccessResponse::SuccessReply(data) => {
                    CString::new(data).unwrap().into_raw()
                }
            }
        }
        Err(_) => {
            CString::new(ERROR_STATE.to_string()).unwrap().into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn smol_db_client_setup_encryption(client_ptr: *mut FFISmolDBClient) -> i32 {
    let client = unsafe {
        assert!(!client_ptr.is_null());
        &mut *client_ptr
    };
    return match client.client.setup_encryption() {
        Ok(_) => {
            OK_STATE
        }
        Err(_) => {
            ERROR_STATE
        }
    } as i32;
}

#[no_mangle]
pub extern "C" fn smol_db_client_reconnect(client_ptr: *mut FFISmolDBClient) -> i32 {
    let client = unsafe {
        assert!(!client_ptr.is_null());
        &mut *client_ptr
    };
    return match client.client.reconnect() {
        Ok(_) => {
            OK_STATE
        }
        Err(_) => {
            ERROR_STATE
        }
    } as i32;
}

#[no_mangle]
pub extern "C" fn smol_db_client_disconnect(client_ptr: *mut FFISmolDBClient) -> i32 {
    let client = unsafe {
        assert!(!client_ptr.is_null());
        &mut *client_ptr
    };
    return match client.client.disconnect() {
        Ok(_) => {
            OK_STATE
        }
        Err(_) => {
            ERROR_STATE
        }
    } as i32;
}

#[no_mangle]
pub extern "C" fn smol_db_client_set_key(client_ptr: *mut FFISmolDBClient, key_ptr: *const c_char) -> i32 {
    let client = unsafe {
        assert!(!client_ptr.is_null());
        &mut *client_ptr
    };
    let key = unsafe {
        assert!(!key_ptr.is_null());
        CStr::from_ptr(key_ptr).to_str().unwrap()
    };
    return match client.client.set_access_key(key.to_string()) {
        Ok(_) => {
            OK_STATE
        }
        Err(_) => {
            ERROR_STATE
        }
    } as i32;
}

#[no_mangle]
pub extern "C" fn smol_db_client_write_db(client_ptr: *mut FFISmolDBClient,name: *const c_char,location: *const c_char,data: *const c_char)
-> *const c_char {

    let client = unsafe {
        assert!(!client_ptr.is_null());
        &mut *client_ptr
    };

    let db_name = unsafe {
        assert!(!name.is_null());
        CStr::from_ptr(name).to_str().unwrap()
    };
    let db_location = unsafe {
        assert!(!location.is_null());
        CStr::from_ptr(location).to_str().unwrap()
    };
    let db_data = unsafe {
        assert!(!data.is_null());
        CStr::from_ptr(data).to_str().unwrap()
    };
    let response = client.client.write_db(db_name,db_location,db_data);

    return match response {
        Ok(success) => {
            match success {
                DBSuccessResponse::SuccessNoData => {
                    CString::new([OK_STATE]).unwrap().into_raw()
                }
                DBSuccessResponse::SuccessReply(data) => {
                    CString::new(data).unwrap().into_raw()
                }
            }
        }
        Err(_) => {
            CString::new([ERROR_STATE]).unwrap().into_raw()
        }
    };
}
