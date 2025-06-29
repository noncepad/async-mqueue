use crate::am::AsyncMQueue;
use crate::{am, mqueue};
use nix::mqueue::MQ_OFlag;
use nix::sys::stat::Mode;
pub const DEFAULT_MESSAGE_SIZE: usize = 8192;
/// Create a mqueue.
pub fn create(
    mpath: String,
    msg_channel_size: Option<usize>,
    msg_size: Option<usize>,
) -> Result<AsyncMQueue, Box<dyn std::error::Error>> {
    inside_open(mpath, true, msg_channel_size, msg_size)
}

/// Open an existing mqueue.
pub fn open(
    mpath: String,
    msg_channel_size: Option<usize>,
    msg_size: Option<usize>,
) -> Result<AsyncMQueue, Box<dyn std::error::Error>> {
    inside_open(mpath, false, msg_channel_size, msg_size)
}

/// Delete a message queue.
pub fn unlink(mpath: String) -> Result<(), Box<dyn std::error::Error>> {
    let name = std::ffi::CString::new(mpath.as_str()).unwrap();
    match mqueue::mq_unlink(&name) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

fn inside_open(
    mpath: String,
    create: bool,
    msg_channel_size: Option<usize>,
    msg_size: Option<usize>,
) -> Result<AsyncMQueue, Box<dyn std::error::Error>> {
    let channel_size = if let Some(x) = msg_channel_size {
        x
    } else {
        10
    };
    let size = if let Some(x) = msg_size {
        assert!(
            x < DEFAULT_MESSAGE_SIZE,
            "max mq size is DEFAULT_MESSAGE_SIZE"
        );
        x
    } else {
        DEFAULT_MESSAGE_SIZE
    };
    //    warn!(
    //       "inside_open - 1 - create {}; channel {:?}; size {:?}",
    //     create, msg_channel_size, msg_size
    //  );
    let attr = Some(mqueue::MqAttr::new(0, channel_size as _, size as _, 0));
    //warn!("inside_open - 2 - create {}", create);
    let name = std::ffi::CString::new(mpath.as_str()).unwrap();
    let mq;
    if create {
        //   warn!("inside_open - 3 - create {}", create);
        mq = mqueue::mq_open(
            &name,
            MQ_OFlag::O_RDWR | MQ_OFlag::O_CREAT | MQ_OFlag::O_NONBLOCK,
            Mode::S_IWUSR | Mode::S_IRUSR | Mode::S_IRGRP | Mode::S_IROTH,
            attr.as_ref(),
        )?;
    } else {
        //  warn!("inside_open - 4 - create {}", create);
        mq = mqueue::mq_open(
            &name,
            MQ_OFlag::O_RDWR | MQ_OFlag::O_NONBLOCK,
            Mode::S_IWUSR | Mode::S_IRUSR | Mode::S_IRGRP | Mode::S_IROTH,
            attr.as_ref(),
        )?;
    }
    //    warn!("inside_open - 5 - create {}", create);
    Ok(am::AsyncMQueue::from(mq))
}
