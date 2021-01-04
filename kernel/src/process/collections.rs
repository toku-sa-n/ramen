// SPDX-License-Identifier: GPL-3.0-or-later

use super::Process;
use alloc::collections::BTreeMap;

static PROCESSES: BTreeMap<super::Id, Process> = BTreeMap::new();
