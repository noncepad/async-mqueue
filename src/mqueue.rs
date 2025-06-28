use nix::errno::Errno;
use nix::Result;

use nix::libc::{self, c_char, mqd_t, size_t};
use nix::mqueue::MQ_OFlag;
use nix::sys::stat::Mode;
use std::ffi::CStr;
use std::mem;

use std::os::unix::io::AsRawFd;

/// A message-queue attribute, optionally used with [`mq_setattr`] and
/// [`mq_getattr`] and optionally [`mq_open`],
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MqAttr {
    mq_attr: libc::mq_attr,
}

#[cfg(all(target_arch = "x86_64"))]
#[cfg_attr(docsrs, doc(cfg(all())))]
pub type MqAttrMemberT = i64;
/// Size of a message queue attribute member
#[cfg(not(all(target_arch = "x86_64")))]
#[cfg_attr(docsrs, doc(cfg(all())))]
pub type MqAttrMemberT = libc::c_long;

impl MqAttr {
    /// Create a new message queue attribute
    ///
    /// # Arguments
    ///
    /// - `mq_flags`:   Either `0` or `O_NONBLOCK`.
    /// - `mq_maxmsg`:  Maximum number of messages on the queue.
    /// - `mq_msgsize`: Maximum message size in bytes.
    /// - `mq_curmsgs`: Number of messages currently in the queue.
    pub fn new(
        mq_flags: MqAttrMemberT,
        mq_maxmsg: MqAttrMemberT,
        mq_msgsize: MqAttrMemberT,
        mq_curmsgs: MqAttrMemberT,
    ) -> MqAttr {
        let mut attr = mem::MaybeUninit::<libc::mq_attr>::uninit();
        unsafe {
            let p = attr.as_mut_ptr();
            (*p).mq_flags = mq_flags;
            (*p).mq_maxmsg = mq_maxmsg;
            (*p).mq_msgsize = mq_msgsize;
            (*p).mq_curmsgs = mq_curmsgs;
            MqAttr {
                mq_attr: attr.assume_init(),
            }
        }
    }

    /// The current flags, either `0` or `O_NONBLOCK`.
    pub const fn flags(&self) -> MqAttrMemberT {
        self.mq_attr.mq_flags
    }

    /// The max number of messages that can be held by the queue
    pub const fn maxmsg(&self) -> MqAttrMemberT {
        self.mq_attr.mq_maxmsg
    }

    /// The maximum size of each message (in bytes)
    pub const fn msgsize(&self) -> MqAttrMemberT {
        self.mq_attr.mq_msgsize
    }

    /// The number of messages currently held in the queue
    pub const fn curmsgs(&self) -> MqAttrMemberT {
        self.mq_attr.mq_curmsgs
    }
}

/// A message-queue attribute, optionally used with [`mq_setattr`] and
/// [`mq_getattr`] and optionally [`mq_open`],
/// Identifies an open POSIX Message Queue
// A safer wrapper around libc::mqd_t, which is a pointer on some platforms
// Deliberately is not Clone to prevent use-after-close scenarios
#[repr(transparent)]
#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct MqdT(mqd_t);

impl AsRawFd for MqdT {
    fn as_raw_fd(&self) -> std::os::fd::RawFd {
        self.0
    }
}

impl MqdT {
    pub fn from(self) -> Self {
        self
    }

    pub fn read(&self, buf: &mut [u8]) -> std::result::Result<usize, std::io::Error> {
        let mut priorit = 0;
        match mq_receive(self, buf, &mut priorit) {
            Ok(len) => Ok(len),
            Err(e) => Err(e.into()),
        }
    }

    pub fn write(&self, buf: &[u8]) -> std::result::Result<usize, std::io::Error> {
        match mq_send(self, buf, 0) {
            Ok(_) => Ok(0),
            Err(e) => Err(e.into()),
        }
    }
}

// The mode.bits cast is only lossless on some OSes
#[allow(clippy::cast_lossless)]
pub fn mq_open(name: &CStr, oflag: MQ_OFlag, mode: Mode, attr: Option<&MqAttr>) -> Result<MqdT> {
    let res = match attr {
        Some(mq_attr) => unsafe {
            libc::mq_open(
                name.as_ptr(),
                oflag.bits(),
                mode.bits() as libc::c_int,
                &mq_attr.mq_attr as *const libc::mq_attr,
            )
        },
        None => unsafe { libc::mq_open(name.as_ptr(), oflag.bits()) },
    };
    Errno::result(res).map(MqdT)
}

/// Remove a message queue
///
/// See also [`mq_unlink(2)`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/mq_unlink.html)
pub fn mq_unlink(name: &CStr) -> Result<()> {
    let res = unsafe { libc::mq_unlink(name.as_ptr()) };
    Errno::result(res).map(drop)
}

/// Close a message queue
///
/// See also [`mq_close(2)`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/mq_close.html)
pub fn mq_close(mqdes: MqdT) -> Result<()> {
    let res = unsafe { libc::mq_close(mqdes.0) };
    Errno::result(res).map(drop)
}

/// Receive a message from a message queue
///
/// See also [`mq_receive(2)`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/mq_receive.html)
pub fn mq_receive(mqdes: &MqdT, message: &mut [u8], msg_prio: &mut u32) -> Result<usize> {
    let len = message.len() as size_t;
    let res = unsafe {
        libc::mq_receive(
            mqdes.0,
            message.as_mut_ptr() as *mut c_char,
            len,
            msg_prio as *mut u32,
        )
    };
    Errno::result(res).map(|r| r as usize)
}

/// Send a message to a message queue
///
/// See also [`mq_send(2)`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/mq_send.html)
pub fn mq_send(mqdes: &MqdT, message: &[u8], msq_prio: u32) -> Result<()> {
    let res = unsafe {
        libc::mq_send(
            mqdes.0,
            message.as_ptr() as *const c_char,
            message.len(),
            msq_prio,
        )
    };
    Errno::result(res).map(drop)
}

/// Get message queue attributes
///
/// See also [`mq_getattr(2)`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/mq_getattr.html)
pub fn mq_getattr(mqd: &MqdT) -> Result<MqAttr> {
    let mut attr = mem::MaybeUninit::<libc::mq_attr>::uninit();
    let res = unsafe { libc::mq_getattr(mqd.0, attr.as_mut_ptr()) };
    Errno::result(res).map(|_| unsafe {
        MqAttr {
            mq_attr: attr.assume_init(),
        }
    })
}

/// Set the attributes of the message queue. Only `O_NONBLOCK` can be set, everything else will be ignored
/// Returns the old attributes
/// It is recommend to use the `mq_set_nonblock()` and `mq_remove_nonblock()` convenience functions as they are easier to use
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/mq_setattr.html)
pub fn mq_setattr(mqd: &MqdT, newattr: &MqAttr) -> Result<MqAttr> {
    let mut attr = mem::MaybeUninit::<libc::mq_attr>::uninit();
    let res = unsafe {
        libc::mq_setattr(
            mqd.0,
            &newattr.mq_attr as *const libc::mq_attr,
            attr.as_mut_ptr(),
        )
    };
    Errno::result(res).map(|_| unsafe {
        MqAttr {
            mq_attr: attr.assume_init(),
        }
    })
}

/// Convenience function.
/// Sets the `O_NONBLOCK` attribute for a given message queue descriptor
/// Returns the old attributes
#[allow(clippy::useless_conversion)] // Not useless on all OSes
pub fn mq_set_nonblock(mqd: &MqdT) -> Result<MqAttr> {
    let oldattr = mq_getattr(mqd)?;
    let newattr = MqAttr::new(
        MqAttrMemberT::from(MQ_OFlag::O_NONBLOCK.bits()),
        oldattr.mq_attr.mq_maxmsg,
        oldattr.mq_attr.mq_msgsize,
        oldattr.mq_attr.mq_curmsgs,
    );
    mq_setattr(mqd, &newattr)
}

/// Convenience function.
/// Removes `O_NONBLOCK` attribute for a given message queue descriptor
/// Returns the old attributes
pub fn mq_remove_nonblock(mqd: &MqdT) -> Result<MqAttr> {
    let oldattr = mq_getattr(mqd)?;
    let newattr = MqAttr::new(
        0,
        oldattr.mq_attr.mq_maxmsg,
        oldattr.mq_attr.mq_msgsize,
        oldattr.mq_attr.mq_curmsgs,
    );
    mq_setattr(mqd, &newattr)
}
