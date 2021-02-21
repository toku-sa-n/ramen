// SPDX-License-Identifier: GPL-3.0-or-later

use uefi::{
    table::{Boot, SystemTable},
    ResultExt,
};

pub fn init(system_table: &SystemTable<Boot>) {
    init_uefi_utils(&system_table);
    reset_console(&system_table);
}

fn init_uefi_utils(system_table: &SystemTable<Boot>) {
    uefi_services::init(system_table).expect_success("Failed to initialize_uefi_utilities");
}

fn reset_console(system_table: &SystemTable<Boot>) {
    system_table
        .stdout()
        .reset(false)
        .expect_success("Failed to reset stdout");
}
