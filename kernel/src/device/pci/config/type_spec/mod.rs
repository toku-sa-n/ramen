// SPDX-License-Identifier: GPL-3.0-or-later

mod non_bridge;

use super::{Bar, RawSpace};

enum TypeSpec {
    NonBridge(non_bridge::TypeSpecNonBridge),
}
