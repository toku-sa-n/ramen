// SPDX-License-Identifier: GPL-3.0-or-later

use common::vram;
use core::mem::MaybeUninit;
use uefi::{
    proto::console::{gop, gop::PixelFormat},
    table::boot,
    ResultExt,
};

#[must_use]
pub fn init(boot_services: &boot::BootServices) -> vram::Info {
    let gop = fetch_gop(boot_services);
    set_resolution(gop);

    vram::Info::new_from_gop(gop)
}

fn fetch_gop<'a>(boot_services: &boot::BootServices) -> &'a mut gop::GraphicsOutput<'a> {
    let gop = boot_services
        .locate_protocol::<gop::GraphicsOutput>()
        .expect_success("Your computer does not support Graphics Output Protocol!");

    unsafe { &mut *gop.get() }
}

fn set_resolution(gop: &mut gop::GraphicsOutput) {
    let (width, height, mode) = get_the_maximum_resolution_and_mode(gop);

    gop.set_mode(&mode)
        .expect_success("Failed to set resolution.");

    info!("width: {} height: {}", width, height);
}

fn get_the_maximum_resolution_and_mode(gop: &gop::GraphicsOutput) -> (usize, usize, gop::Mode) {
    let mut max_height = 0;
    let mut max_width = 0;
    let mut preferred_mode = MaybeUninit::<gop::Mode>::uninit();

    for mode in gop.modes() {
        let mode = mode.expect("Failed to get gop mode.");

        let (width, height) = mode.info().resolution();
        if height > max_height && width > max_width && is_usable_gop_mode(&mode.info()) {
            max_height = height;
            max_width = width;
            unsafe { preferred_mode.as_mut_ptr().write(mode) }
        }
    }

    (max_height, max_width, unsafe {
        preferred_mode.assume_init()
    })
}

fn is_usable_gop_mode(mode: &gop::ModeInfo) -> bool {
    if mode.pixel_format() != PixelFormat::Bgr {
        return false;
    }

    // According to UEFI Specification 2.8 Errata A, P.479,
    // . : Pixel
    // P : Padding
    // ..........................................PPPPPPPPPP
    // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^|^^^^^^^^^^
    //             HorizontalResolution         | Paddings
    // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    //                    PixelsPerScanLine
    //
    // This OS doesn't deal with pixel paddings, so return an error if pixel paddings exist.
    let (width, _) = mode.resolution();
    if width != mode.stride() {
        return false;
    }

    true
}
