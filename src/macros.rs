macro_rules! forward_read {
    ($t:ty) => {
        impl std::io::Read for $t {
            #[inline]
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                self.0.read(buf)
            }
            #[inline]
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
            #[inline]
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0.write(buf)
            }
            #[inline]
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
