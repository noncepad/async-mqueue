pub mod am;
pub mod mqueue;
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    // Import the `tokio::test` macro
    use super::*;
    use log::warn;
    use nix::mqueue::MQ_OFlag;
    use nix::sys::stat::Mode;
    use tokio::test;
    use tokio::time::sleep;

    #[test]
    async fn it_works() -> Result<(), Box<dyn std::error::Error>> {
        env_logger::init().unwrap();
        let mut handlers = vec![];
        let mpath = String::from("/mqueue2");
        let start = 10;
        {
            let s1 = start;
            let m1 = mpath.clone();
            handlers.push(tokio::spawn(async move {
                recv(s1, m1).await.unwrap();
            }));
        }
        {
            let s1 = start;
            let m1 = mpath.clone();
            handlers.push(tokio::spawn(async move {
                send(s1, m1).await.unwrap();
            }));
        }
        for handler in handlers {
            handler.await.unwrap();
        }

        Ok(())
    }

    async fn recv(start: usize, mpath: String) -> Result<(), Box<dyn std::error::Error>> {
        let attr = Some(mqueue::MqAttr::new(0, 10, 4096, 0));
        let name = std::ffi::CString::new(mpath.as_str()).unwrap();
        let mq = mqueue::mq_open(
            &name,
            MQ_OFlag::O_RDWR | MQ_OFlag::O_CREAT | MQ_OFlag::O_NONBLOCK,
            Mode::S_IWUSR | Mode::S_IRUSR | Mode::S_IRGRP | Mode::S_IROTH,
            attr.as_ref(),
        )?;

        let mq = am::AsyncMQueue::from(mq);

        let mut data = [0; 4096];
        let mut i = start;
        while 0 < i {
            i -= 1;
            let size = mq.read(&mut data).await?;
            warn!("receiving bytes {}; i {}", size, i);
        }
        Ok(())
    }

    async fn send(start: usize, mpath: String) -> Result<(), Box<dyn std::error::Error>> {
        let attr = Some(mqueue::MqAttr::new(0, 10, 4096, 0));
        let name = std::ffi::CString::new(mpath.as_str()).unwrap();
        let mq = mqueue::mq_open(
            &name,
            MQ_OFlag::O_RDWR | MQ_OFlag::O_CREAT | MQ_OFlag::O_NONBLOCK,
            Mode::S_IWUSR | Mode::S_IRUSR | Mode::S_IRGRP | Mode::S_IROTH,
            attr.as_ref(),
        )?;

        let mq = am::AsyncMQueue::from(mq);

        let mut data = [0; 16];
        let mut i = start;
        while 0 < i {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            warn!("sending i {}", i);
            mq.write(&mut data).await?;
            i -= 1;
        }
        sleep(Duration::from_secs(10)).await;
        mqueue::mq_unlink(&name)?;
        Ok(())
    }
}
