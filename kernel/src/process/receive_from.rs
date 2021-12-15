use super::Pid;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) enum ReceiveFrom {
    Any,
    Id(Pid),
}
