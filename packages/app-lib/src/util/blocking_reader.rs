// Adapted from https://docs.rs/crate/tokio/1.26.0/source/src/io/blocking.rs

use std::{io::Read, pin::Pin, task::{Context, Poll, ready}};

use tokio::{io::{AsyncRead, ReadBuf}, task::JoinHandle};

/// `T` should not implement _both_ Read and Write.
#[derive(Debug)]
pub(crate) struct Blocking<T> {
    inner: Option<T>,
    state: State<T>
}

#[derive(Debug)]
pub(crate) struct Buf {
    buf: Vec<u8>,
    pos: usize,
}

pub(crate) const MAX_BUF: usize = 2 * 1024 * 1024;

#[derive(Debug)]
enum State<T> {
    Idle(Option<Buf>),
    Busy(JoinHandle<(std::io::Result<usize>, Buf, T)>),
}

impl<T> Blocking<T> {
    pub(crate) fn new(inner: T) -> Blocking<T> {
        Blocking {
            inner: Some(inner),
            state: State::Idle(Some(Buf::with_capacity(0))),
        }
    }
}

impl<T> AsyncRead for Blocking<T>
where
    T: Read + Unpin + Send + 'static,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        dst: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        loop {
            match self.state {
                State::Idle(ref mut buf_cell) => {
                    let mut buf = buf_cell.take().unwrap();

                    if !buf.is_empty() {
                        buf.copy_to(dst);
                        *buf_cell = Some(buf);
                        return Poll::Ready(Ok(()));
                    }

                    buf.ensure_capacity_for(dst);
                    let mut inner = self.inner.take().unwrap();

                    self.state = State::Busy(tokio::task::spawn_blocking(move || {
                        let res = buf.read_from(&mut inner);
                        (res, buf, inner)
                    }));
                }
                State::Busy(ref mut rx) => {
                    let (res, mut buf, inner) = ready!(Pin::new(rx).poll(cx))?;
                    self.inner = Some(inner);

                    match res {
                        Ok(_) => {
                            buf.copy_to(dst);
                            self.state = State::Idle(Some(buf));
                            return Poll::Ready(Ok(()));
                        }
                        Err(e) => {
                            assert!(buf.is_empty());

                            self.state = State::Idle(Some(buf));
                            return Poll::Ready(Err(e));
                        }
                    }
                }
            }
        }
    }
}

/// Repeats operations that are interrupted.
macro_rules! uninterruptibly {
    ($e:expr) => {{
        loop {
            match $e {
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                res => break res,
            }
        }
    }};
}

impl Buf {
    pub(crate) fn with_capacity(n: usize) -> Buf {
        Buf {
            buf: Vec::with_capacity(n),
            pos: 0,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn len(&self) -> usize {
        self.buf.len() - self.pos
    }

    pub(crate) fn copy_to(&mut self, dst: &mut ReadBuf<'_>) -> usize {
        let n = std::cmp::min(self.len(), dst.remaining());
        dst.put_slice(&self.bytes()[..n]);
        self.pos += n;

        if self.pos == self.buf.len() {
            self.buf.truncate(0);
            self.pos = 0;
        }

        n
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.buf[self.pos..]
    }

    pub(crate) fn ensure_capacity_for(&mut self, bytes: &ReadBuf<'_>) {
        assert!(self.is_empty());

        let len = std::cmp::min(bytes.remaining(), MAX_BUF);

        if self.buf.len() < len {
            self.buf.reserve(len - self.buf.len());
        }

        unsafe {
            self.buf.set_len(len);
        }
    }

    pub(crate) fn read_from<T: Read>(&mut self, rd: &mut T) -> std::io::Result<usize> {
        let res = uninterruptibly!(rd.read(&mut self.buf));

        if let Ok(n) = res {
            self.buf.truncate(n);
        } else {
            self.buf.clear();
        }

        assert_eq!(self.pos, 0);

        res
    }
}
