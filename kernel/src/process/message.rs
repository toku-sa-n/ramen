// SPDX-License-Identifier: GPL-3.0-or-later

pub(super) struct Message {
    header: Header,
    body: Body,
}

pub(super) struct Header {
    sender: super::Id,
}

pub(super) struct Body {
    m1: u64,
    m2: u64,
    m3: u64,
}
