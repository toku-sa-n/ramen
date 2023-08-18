// SPDX-License-Identifier: GPL-3.0-or-later

use {
    boot_info::vram,
    core::mem::MaybeUninit,
    log::info,
    uefi::{
        proto::console::{gop, gop::PixelFormat},
        table::{boot, boot::ScopedProtocol},
    },
    vek::Vec2,
    x86_64::PhysAddr,
};

// Not too big, not too small, just a good resolution.
const RESOLUTION: Vec2<usize> = Vec2::new(800, 600);

#[must_use]
pub fn init(boot_services: &boot::BootServices) -> vram::Info {
    let mut gop = fetch_gop(boot_services);
    set_resolution(&mut gop);

    gop_to_boot_info(&mut gop)
}

fn fetch_gop(boot_services: &boot::BootServices) -> ScopedProtocol<'_, gop::GraphicsOutput> {
    let handle = boot_services
        .get_handle_for_protocol::<gop::GraphicsOutput>()
        .expect("Failed to get handle for GOP protocol!");

    boot_services
        .open_protocol_exclusive(handle)
        .expect("Failed to open GOP protocol!")
}

fn set_resolution(gop: &mut gop::GraphicsOutput) {
    let mode = get_mode(gop);

    gop.set_mode(&mode).expect("Failed to set resolution.");
}

fn gop_to_boot_info(gop: &mut gop::GraphicsOutput) -> vram::Info {
    let resolution: Vec2<usize> = gop.current_mode_info().resolution().into();

    assert_eq!(resolution, RESOLUTION);

    vram::Info::new(
        32,
        resolution.as_(),
        PhysAddr::new(gop.frame_buffer().as_mut_ptr() as u64),
    )
}

fn get_mode(gop: &gop::GraphicsOutput) -> gop::Mode {
    for mode in gop.modes() {
        let (width, height) = mode.info().resolution();
        if width == RESOLUTION.x && height == RESOLUTION.y && is_usable_gop_mode(mode.info()) {
            return mode;
        }
    }

    todo!()
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
