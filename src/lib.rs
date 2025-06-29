pub mod am;
pub mod mqueue;
pub mod simple;
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use self::simple::DEFAULT_MESSAGE_SIZE;

    // Import the `tokio::test` macro
    use super::*;
    use log::info;
    use tokio::test;
    use tokio::time::sleep;

    #[test]
    async fn it_works() -> Result<(), Box<dyn std::error::Error>> {
        env_logger::init().unwrap();
        let mut handlers = vec![];
        let mpath = String::from("/mqueue0");
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
        info!("recv - 1");
        sleep(Duration::from_secs(2)).await;
        let mq = simple::open(mpath, None, None)?;
        info!("recv - 2");
        let mut data = [0; DEFAULT_MESSAGE_SIZE];
        let mut i = start;
        while 0 < i {
            i -= 1;
            let size = mq.read(&mut data).await?;
            info!("receiving bytes {}; i {}", size, i);
        }
        info!("recv - 3");
        Ok(())
    }

    async fn send(start: usize, mpath: String) -> Result<(), Box<dyn std::error::Error>> {
        info!("send - 1");
        let mq = simple::create(mpath.clone(), None, None)?;
        info!("send - 2");
        let mut data = [0; 16];
        let mut i = start;
        while 0 < i {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            info!("sending i {}", i);
            mq.write(&mut data).await?;
            i -= 1;
        }
        info!("send - 3");
        sleep(Duration::from_secs(10)).await;
        simple::unlink(mpath)?;
        info!("send - 4");
        Ok(())
    }
}
