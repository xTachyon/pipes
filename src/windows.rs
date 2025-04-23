// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

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
