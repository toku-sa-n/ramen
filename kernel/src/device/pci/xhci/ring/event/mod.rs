// SPDX-License-Identifier: GPL-3.0-or-later

use super::Raw;

mod segment_table;

struct EventRing<'a> {
    raw: Raw<'a>,
}
