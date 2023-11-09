use std::ffi::{c_char, CStr};
use smol_db_client::{DBSuccessResponse, SmolDbClient};

#[no_mangle]
pub extern "C" fn smol_db_client_new(ip: *const c_char) -> *mut SmolDbClient {
    let ip_address = unsafe {
        assert!(!ip.is_null());
        CStr::from_ptr(ip).to_str().unwrap()
    };

    // TODO: fix this unwrap ?
    Box::into_raw(Box::new(SmolDbClient::new(ip_address).unwrap()))
}

#[no_mangle]
pub extern "C" fn smol_db_client_free(client_ptr: *mut SmolDbClient) {
    if client_ptr.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(client_ptr);
    }
}

#[no_mangle]
pub extern "C" fn smol_db_client_set_key(client_ptr: *mut SmolDbClient,key_ptr: *const c_char) {
    let client = unsafe {
        assert!(!client_ptr.is_null());
        &mut *client_ptr
    };
    let key = unsafe {
        assert!(!key_ptr.is_null());
        CStr::from_ptr(key_ptr).to_str().unwrap()
    };
    client.set_access_key(key.to_string());

}

#[no_mangle]
pub extern "C" fn smol_db_client_write_db(client_ptr: *mut SmolDbClient,name: *const c_char,location: *const c_char,data: *const c_char)
-> *mut u8 {

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
    let response = client.write_db(db_name,db_location,db_data);

    let response_data = match response {
        Ok(success) => {
            match success {
                DBSuccessResponse::SuccessNoData => {
                    Box::new([0u8])
                }
                DBSuccessResponse::SuccessReply(output_data) => {
                    Box::new([2u8])
                }
            }
        }
        Err(err) => {
            Box::new([1u8])
        }
    };

    Box::into_raw(response_data) as *mut _
}
