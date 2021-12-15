use super::Pid;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub(crate) enum ReceiveFrom {
    Any,
    Id(Pid),
}
