pub struct OS;

impl OS {
    pub fn err_no() -> i32 {
        let errno = unsafe { *libc::__errno_location() };
        return errno;
    }

    pub fn err_msg() -> String {
        let errno = OS::err_no();
        let cstr = unsafe { std::ffi::CStr::from_ptr(libc::strerror(errno)) };
        cstr.to_string_lossy().into_owned()
    }
}
