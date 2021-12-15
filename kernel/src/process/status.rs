use {
    super::{receive_from::ReceiveFrom, Pid},
    x86_64::PhysAddr,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Status {
    Running,
    Runnable,
    Sending { to: Pid, message: PhysAddr },
    Receiving(ReceiveFrom),
}
