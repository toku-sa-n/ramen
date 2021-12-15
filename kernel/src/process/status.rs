use {
    super::{receive_from::ReceiveFrom, Pid},
    message::Message,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum Status {
    Running,
    Runnable,
    Sending { to: Pid, message: Message },
    Receiving(ReceiveFrom),
}
