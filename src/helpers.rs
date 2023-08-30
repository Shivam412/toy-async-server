use std::error::Error;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn get_error_message() -> String {
    unsafe {
        let errno = *libc::__errno_location();
        let cstr = std::ffi::CStr::from_ptr(libc::strerror(errno));
        cstr.to_string_lossy().into_owned()
    }
}
