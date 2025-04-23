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

pub use os::Recver;
pub use os::Sender;

use anyhow::anyhow;
use anyhow::Result;

pub struct DuplexPipe {
    pub r: os::Recver,
    pub s: os::Sender,
}

pub struct DuplexPipeToSend {
    r: os::OwnedThingy,
    s: os::OwnedThingy,
}
impl DuplexPipeToSend {
    pub fn to_string(&self) -> String {
        format!(
            "dpipe:{},{}",
            os::to_string(&self.r),
            os::to_string(&self.s)
        )
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
    assert_eq!(split.next(), None);

    let mut r = os::from_string(r)?;
    let mut s = os::from_string(s)?;

    unsafe {
        r = set_non_inheritable(r)?;
        s = set_non_inheritable(s)?;
    }

    Ok(DuplexPipe {
        r: r.into(),
        s: s.into(),
    })
}

macro_rules! forward_read {
    ($t:ty) => {
        impl std::io::Read for $t {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                self.0.read(buf)
            }
            fn read_vectored(
                &mut self,
                bufs: &mut [std::io::IoSliceMut<'_>],
            ) -> std::io::Result<usize> {
                self.0.read_vectored(bufs)
            }
        }
    };
}
pub(crate) use forward_read;

macro_rules! forward_write {
    ($t:ty) => {
        impl std::io::Write for $t {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0.write(buf)
            }
            fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
                self.0.write_vectored(bufs)
            }
            #[inline]
            fn flush(&mut self) -> std::io::Result<()> {
                self.0.flush()
            }
        }
    };
}
pub(crate) use forward_write;
