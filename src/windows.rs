use std::os::windows::io::FromRawHandle;
use std::{ffi::c_void, os::windows::io::AsRawHandle};

pub type OwnedThingy = std::os::windows::io::OwnedHandle;

pub fn to_string(x: &OwnedThingy) -> String {
    (x.as_raw_handle() as isize).to_string()
}
pub unsafe fn from_string(x: &str) -> Result<OwnedThingy> {
    let x: isize = x.parse()?;
    Ok(OwnedThingy::from_raw_handle(x as *mut c_void))
}
pub unsafe fn set_inheritable(x: &OwnedThingy) -> Result<()> {
    if windows_sys::Win32::Foundation::SetHandleInformation(
        x.as_raw_handle() as _,
        windows_sys::Win32::Foundation::HANDLE_FLAG_INHERIT,
        windows_sys::Win32::Foundation::HANDLE_FLAG_INHERIT,
    ) == 0
    {
        return Err(anyhow!("failed to set inheritable (SetHandleInformation)"));
    }
    Ok(())
}
pub unsafe fn set_non_inheritable(x: &OwnedThingy) -> Result<()> {
    if windows_sys::Win32::Foundation::SetHandleInformation(
        x.as_raw_handle() as _,
        windows_sys::Win32::Foundation::HANDLE_FLAG_INHERIT,
        0,
    ) == 0
    {
        return Err(anyhow!(
            "failed to set non-inheritable (SetHandleInformation)"
        ));
    }
    Ok(())
}
