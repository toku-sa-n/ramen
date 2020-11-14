// SPDX-License-Identifier: GPL-3.0-or-later

mod ahc;

use ahc::Ahc;

pub async fn task() {
    let mut ahc = match Ahc::new() {
        Some(ahc) => ahc,
        None => return,
    };
    ahc.get_ownership_from_bios();
}
