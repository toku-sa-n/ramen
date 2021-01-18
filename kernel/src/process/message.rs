// SPDX-License-Identifier: GPL-3.0-or-later

use super::collections;

pub(super) fn try_send(to: super::Id, m: Message) -> Result<(), Error> {
    collections::process::handle_running_mut(|p| p.inbox.push(m).map_err(Error::Full))
}

pub(super) fn try_receive() -> Option<Message> {
    let id = super::manager::getpid();
    collections::process::handle_mut(id.into(), |p| p.inbox.pop())
}

#[derive(Debug)]
pub(super) struct Message {
    header: Header,
    body: Body,
}
impl Message {
    pub(super) fn new(header: Header, body: Body) -> Self {
        Self { header, body }
    }
}

#[derive(Debug)]
pub(super) struct Header {
    from: super::Id,
}
impl Header {
    pub(super) fn new(from: super::Id) -> Self {
        Self { from }
    }
}

#[derive(Debug, Default)]
pub(super) struct Body {
    m1: u64,
    m2: u64,
    m3: u64,
}
impl Body {
    pub(super) fn new(m1: u64, m2: u64, m3: u64) -> Self {
        Self { m1, m2, m3 }
    }
}

#[derive(Debug)]
enum Error {
    Full(Message),
}
