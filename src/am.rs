use crate::mqueue::MqdT;
use futures::ready;
use std::task::Poll;
use tokio::io::unix::AsyncFd;
use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;

pub struct AsyncMQueue {
    inner: AsyncFd<MqdT>,
}

impl AsyncMQueue {
    pub fn from(inner: MqdT) -> Self {
        Self {
            inner: AsyncFd::new(inner).unwrap(),
        }
    }

    pub async fn read(&self, out: &mut [u8]) -> std::io::Result<usize> {
        loop {
            let mut guard = self.inner.readable().await?;

            match guard.try_io(|inner| inner.get_ref().read(out)) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }

    pub async fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
        loop {
            let mut guard = self.inner.writable().await?;

            match guard.try_io(|inner| inner.get_ref().write(buf)) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }
}

impl AsyncRead for AsyncMQueue {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        loop {
            let mut guard = ready!(self.inner.poll_read_ready(cx))?;

            let unfilled = buf.initialize_unfilled();

            match guard.try_io(|inner| inner.get_ref().read(unfilled)) {
                Ok(Ok(len)) => {
                    buf.advance(len);
                    return Poll::Ready(Ok(()));
                }

                Ok(Err(err)) if err.kind() == std::io::ErrorKind::WouldBlock => continue,

                Ok(Err(err)) => return Poll::Ready(Err(err)),

                Err(_would_block) => continue,
            }
        }
    }
}

impl AsyncWrite for AsyncMQueue {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        loop {
            let mut guard = ready!(self.inner.poll_write_ready(cx))?;

            match guard.try_io(|inner| inner.get_ref().write(buf)) {
                Ok(Ok(size)) => return Poll::Ready(Ok(size)),

                Ok(Err(err)) if err.kind() == std::io::ErrorKind::WouldBlock => continue,

                Ok(Err(err)) => return Poll::Ready(Err(err)),

                Err(_would_block) => continue,
            }
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }
}
