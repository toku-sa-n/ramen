// SPDX-License-Identifier: GPL-3.0-or-later

use super::Process;
use alloc::collections::{BTreeMap, VecDeque};
use conquer_once::spin::Lazy;

static PROCESSES: BTreeMap<super::Id, Process> = BTreeMap::new();
static PIDS: Lazy<VecDeque<super::Id>> = Lazy::new(|| VecDeque::new());
