use std::io::Error;
use libc::{__errno_location, listen};

/// wrapper for listen
pub fn safe_listen(socket_fd: i32,max_queue_size: i32) -> std::io::Result<i32> {

    let result = unsafe{
        listen(socket_fd, max_queue_size)
    };

    wrap_result(result)

}

/// Function that extracts errno safely
pub fn get_errno() -> i32{

    let mut errno = 0;

    unsafe{
        errno = *__errno_location();
    }

    errno

}

/// wrapper function for nonnegative values
pub fn wrap_result(result: i32) -> std::io::Result<i32> {

    if result > 0{
        Ok(result)
    }
    else{
        Err(Error::from_raw_os_error(get_errno()))
    }

}