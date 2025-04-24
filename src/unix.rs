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

use crate::{forward_read, forward_write, DuplexPipe, DuplexPipeToSend};
use anyhow::Result;
use std::net::TcpStream;
use std::os::fd::OwnedFd;
use std::os::fd::{AsRawFd, FromRawFd};

fn cvt(t: i32) -> std::io::Result<i32> {
    if t == -1 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(t)
    }
}

pub struct Recver(pub TcpStream);

forward_read!(Recver);

impl From<OwnedFd> for Recver {
    fn from(x: OwnedFd) -> Self {
        Self(x.into())
    }
}

pub struct Sender(pub TcpStream);

forward_write!(Sender);

impl From<OwnedFd> for Sender {
    fn from(x: OwnedFd) -> Self {
        Self(x.into())
    }
}

pub type OwnedThingy = std::os::fd::OwnedFd;

pub fn to_string(x: &OwnedThingy) -> String {
    x.as_raw_fd().to_string()
}

pub unsafe fn from_string(x: &str) -> Result<OwnedThingy> {
    let x: i32 = x.parse()?;
    Ok(OwnedThingy::from_raw_fd(x))
}

pub unsafe fn set_non_inheritable(x: &OwnedThingy) -> Result<()> {
    cvt(libc::fcntl(x.as_raw_fd(), libc::F_SETFD, libc::FD_CLOEXEC))?;
    Ok(())
}

unsafe fn dup(x: &OwnedFd) -> Result<OwnedFd> {
    let fd = cvt(libc::dup(x.as_raw_fd()))?;
    Ok(OwnedFd::from_raw_fd(fd))
}

#[cfg(target_os = "macos")]
fn setsockopt<T>(sock: i32, level: i32, option_name: i32, option_value: T) -> Result<()> {
    unsafe {
        cvt(libc::setsockopt(
            sock,
            level,
            option_name,
            &option_value as *const _ as *const _,
            size_of::<T>() as libc::socklen_t,
        ))?;
        Ok(())
    }
}

pub fn duplex_pipe() -> Result<(DuplexPipe, DuplexPipeToSend)> {
    // #[cfg(target_os = "macos")]
    // const SOCK_CLOEXEC: i32 = 0;
    // #[cfg(not(target_os = "macos"))]
    // const SOCK_CLOEXEC: i32 = libc::SOCK_CLOEXEC;

    let mut sv = [0; 2];

    cvt(unsafe { libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, sv.as_mut_ptr()) })?;

    let fd1 = unsafe { OwnedFd::from_raw_fd(sv[0]) };
    let fd2 = unsafe { OwnedFd::from_raw_fd(sv[1]) };

    #[cfg(target_os = "macos")]
    {
        setsockopt(fd1.as_raw_fd(), libc::SOL_SOCKET, libc::SO_NOSIGPIPE, 1)?;
        setsockopt(fd2.as_raw_fd(), libc::SOL_SOCKET, libc::SO_NOSIGPIPE, 1)?;
    }

    let fd3 = unsafe { dup(&fd1)? };
    let fd4 = unsafe { dup(&fd2)? };

    let mut dpipe = DuplexPipe {
        r: Recver(fd1.into()),
        s: Sender(fd3.into()),
    };
    let dpipe_to_send = DuplexPipeToSend { r: fd2, s: fd4 };

    unsafe {
        dpipe.r = Recver(super::set_non_inheritable(dpipe.r.0)?);
        dpipe.s = Sender(super::set_non_inheritable(dpipe.s.0)?);
    }

    Ok((dpipe, dpipe_to_send))
}
