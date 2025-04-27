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

#[cfg(not(target_os = "windows"))]
mod unix;
#[cfg(target_os = "windows")]
mod windows;
mod os {
    #[cfg(not(target_os = "windows"))]
    pub use super::unix::*;
    #[cfg(target_os = "windows")]
    pub use super::windows::*;
}
mod macros;

use anyhow::anyhow;
use anyhow::Result;
use macros::forward_read;
use macros::forward_write;

pub struct Recver(os::Pipe);
pub struct Sender(os::Pipe);

forward_read!(Recver);
forward_write!(Sender);

pub struct DuplexPipe {
    pub r: Recver,
    pub s: Sender,
}

/// This is used to send the duplex pipe to a child process, usually.
/// Use `with_fds` to spawn a child process with the string given as argument.
/// Then, call duplex_pipe_from_string with the string in the child process.
pub struct DuplexPipeToSend {
    r: os::OwnedThingy,
    s: os::OwnedThingy,
}
impl DuplexPipeToSend {
    pub fn with_fds<F, T, E>(self, f: F) -> Result<T, E>
    where
        F: FnOnce(String) -> Result<T, E>,
    {
        let s = format!(
            "dpipe:{},{}",
            os::to_string(&self.r),
            os::to_string(&self.s)
        );
        f(s)
    }
}

unsafe fn set_non_inheritable<T: Into<os::OwnedThingy> + From<os::OwnedThingy>>(x: T) -> Result<T> {
    let x: os::OwnedThingy = x.into();
    os::set_non_inheritable(&x)?;
    Ok(x.into())
}

pub fn duplex_pipe() -> Result<(DuplexPipe, DuplexPipeToSend)> {
    os::duplex_pipe()
}

/// # Safety
/// This function must be called exactly once in either the same process where
/// DuplexPipeToSend::with_fds was called, or a child process that kept the descriptors open.
/// Calling it more than once on the same string will result in possibly using some already
/// opened file descriptor, which will probably corrupt your data.
/// Not calling it at all will result in a file descriptor leak.
pub unsafe fn duplex_pipe_from_string(name: &str) -> Result<DuplexPipe> {
    let Some(name) = name.strip_prefix("dpipe:") else {
        return Err(anyhow!("duple pipe name must start with dpipe:"));
    };
    let mut split = name.split(",");
    let Some(r) = split.next() else {
        return Err(anyhow!("can't parse reader fd"));
    };
    let Some(s) = split.next() else {
        return Err(anyhow!("can't parse sender fd"));
    };
    if split.next().is_some() {
        return Err(anyhow!("too many arguments in duplex pipe name"));
    }

    let mut r = os::from_string(r)?;
    let mut s = os::from_string(s)?;

    unsafe {
        r = set_non_inheritable(r)?;
        s = set_non_inheritable(s)?;
    }

    Ok(DuplexPipe {
        r: Recver(r.into()),
        s: Sender(s.into()),
    })
}
