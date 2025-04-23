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

use crate::forward_read;
use crate::forward_write;
use crate::DuplexPipe;
use crate::DuplexPipeToSend;
use anyhow::anyhow;
use anyhow::Result;
use std::fs::File;
use std::io;
use std::os::windows::io::FromRawHandle;
use std::os::windows::prelude::OwnedHandle;
use std::ptr::null_mut;
use std::{ffi::c_void, os::windows::io::AsRawHandle};
use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::System::Pipes::CreatePipe;

pub type OwnedThingy = std::os::windows::io::OwnedHandle;

pub struct Recver(File);

forward_read!(Recver);

impl From<OwnedThingy> for Recver {
    fn from(x: OwnedThingy) -> Self {
        Self(x.into())
    }
}

pub struct Sender(File);

forward_write!(Sender);

impl From<OwnedThingy> for Sender {
    fn from(x: OwnedThingy) -> Self {
        Self(x.into())
    }
}

pub fn to_string(x: &OwnedThingy) -> String {
    (x.as_raw_handle() as isize).to_string()
}
pub unsafe fn from_string(x: &str) -> Result<OwnedThingy> {
    let x: isize = x.parse()?;
    Ok(OwnedThingy::from_raw_handle(x as *mut c_void))
}

unsafe fn set_inheritable<T: Into<OwnedThingy> + From<OwnedThingy>>(x: T) -> Result<T> {
    let x: OwnedThingy = x.into();
    set_non_inheritable_impl(&x)?;
    Ok(x.into())
}

pub unsafe fn set_non_inheritable(x: &OwnedThingy) -> Result<()> {
    if windows_sys::Win32::Foundation::SetHandleInformation(
        x.as_raw_handle() as _,
        windows_sys::Win32::Foundation::HANDLE_FLAG_INHERIT,
        0,
    ) == 0
    {
        return Err(anyhow!("failed to set inheritable (SetHandleInformation)"));
    }
    Ok(())
}
pub unsafe fn set_non_inheritable_impl(x: &OwnedThingy) -> Result<()> {
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

fn pipe() -> Result<(OwnedHandle, OwnedHandle)> {
    let mut r = INVALID_HANDLE_VALUE;
    let mut s = INVALID_HANDLE_VALUE;

    let success = unsafe { CreatePipe(&mut r, &mut s, null_mut(), 0) } != 0;
    if !success {
        return Err(io::Error::last_os_error().into());
    }

    let r = unsafe { OwnedHandle::from_raw_handle(r) };
    let s = unsafe { OwnedHandle::from_raw_handle(s) };

    Ok((r, s))
}

pub fn duplex_pipe() -> Result<(DuplexPipe, DuplexPipeToSend)> {
    let (rx_1, tx_1) = pipe()?;
    let (rx_2, tx_2) = pipe()?;

    let dpipe = DuplexPipe {
        r: rx_2.into(),
        s: tx_1.into(),
    };
    let mut dpipe_to_send = DuplexPipeToSend {
        r: rx_1.into(),
        s: tx_2.into(),
    };

    unsafe {
        dpipe_to_send.r = set_inheritable(dpipe_to_send.r)?;
        dpipe_to_send.s = set_inheritable(dpipe_to_send.s)?;
    }

    Ok((dpipe, dpipe_to_send))
}
