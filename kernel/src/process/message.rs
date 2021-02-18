// SPDX-License-Identifier: GPL-3.0-or-later

use super::collections::process;

#[derive(Debug)]
pub(super) struct Message;

pub(super) fn send(to: super::Id, m: Message) {
    process::handle_mut(to, |p| p.inbox.push(m).expect("Inbox is full."))
}

pub(super) fn try_receive() -> Option<Message> {
    process::handle_running(|p| p.inbox.pop())
}
