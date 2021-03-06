use super::*;
use util::ring_buf::*;

// TODO: Use Waiter and WaitQueue infrastructure to sleep when blocking

pub const PIPE_BUF_SIZE: usize = 2 * 1024 * 1024;

#[derive(Debug)]
pub struct Pipe {
    pub reader: PipeReader,
    pub writer: PipeWriter,
}

impl Pipe {
    pub fn new() -> Result<Pipe> {
        let mut ring_buf = RingBuf::new(PIPE_BUF_SIZE);
        Ok(Pipe {
            reader: PipeReader {
                inner: SgxMutex::new(ring_buf.reader),
            },
            writer: PipeWriter {
                inner: SgxMutex::new(ring_buf.writer),
            },
        })
    }
}

#[derive(Debug)]
pub struct PipeReader {
    inner: SgxMutex<RingBufReader>,
}

impl File for PipeReader {
    fn read(&self, buf: &mut [u8]) -> Result<usize> {
        let ringbuf = self.inner.lock().unwrap();
        ringbuf.read(buf)
    }

    fn readv(&self, bufs: &mut [&mut [u8]]) -> Result<usize> {
        let mut ringbuf = self.inner.lock().unwrap();
        let mut total_bytes = 0;
        for buf in bufs {
            match ringbuf.read(buf) {
                Ok(this_len) => {
                    total_bytes += this_len;
                    if this_len < buf.len() {
                        break;
                    }
                }
                Err(e) => {
                    match total_bytes {
                        // a complete failure
                        0 => return Err(e),
                        // a partially failure
                        _ => break,
                    }
                }
            }
        }
        Ok(total_bytes)
    }

    fn as_any(&self) -> &Any {
        self
    }
}

unsafe impl Send for PipeReader {}
unsafe impl Sync for PipeReader {}

#[derive(Debug)]
pub struct PipeWriter {
    inner: SgxMutex<RingBufWriter>,
}

impl File for PipeWriter {
    fn write(&self, buf: &[u8]) -> Result<usize> {
        let ringbuf = self.inner.lock().unwrap();
        ringbuf.write(buf)
    }

    fn writev(&self, bufs: &[&[u8]]) -> Result<usize> {
        let ringbuf = self.inner.lock().unwrap();
        let mut total_bytes = 0;
        for buf in bufs {
            match ringbuf.write(buf) {
                Ok(this_len) => {
                    total_bytes += this_len;
                    if this_len < buf.len() {
                        break;
                    }
                }
                Err(e) => {
                    match total_bytes {
                        // a complete failure
                        0 => return Err(e),
                        // a partially failure
                        _ => break,
                    }
                }
            }
        }
        Ok(total_bytes)
    }

    fn seek(&self, pos: SeekFrom) -> Result<off_t> {
        return_errno!(ESPIPE, "Pipe does not support seek")
    }

    fn as_any(&self) -> &Any {
        self
    }
}

unsafe impl Send for PipeWriter {}
unsafe impl Sync for PipeWriter {}
