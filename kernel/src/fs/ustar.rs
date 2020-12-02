// SPDX-License-Identifier: GPL-3.0-or-later

#[repr(C, packed)]
struct Meta {
    name: [u8; 100],
    mode: [u8; 8],
    owner: [u8; 8],
    group: [u8; 8],
    size: [u8; 12],
    modified_time: [u8; 12],
    checksum: [u8; 8],
    ty: [u8; 1],
    linked_file_name: [u8; 100],
    magic: [u8; 6],
    version: [u8; 2],
    owner_name: [u8; 32],
    group_name: [u8; 32],
    device_major_number: [u8; 8],
    device_minor_number: [u8; 8],
    filename_prefix: [u8; 155],
    _rsvd: [u8; 12],
}
