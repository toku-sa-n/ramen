// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::edid,
    boot_info::vram,
    log::info,
    uefi::{
        proto::console::{gop, gop::PixelFormat},
        table::{boot, boot::ScopedProtocol},
    },
    vek::Vec2,
    x86_64::PhysAddr,
};

fn stop() {
    loop {
        unsafe { core::arch::asm!("mov rax, 0x55aa55aa;cli;hlt") }
    }
}

#[must_use]
pub fn init(boot_services: &boot::BootServices) -> vram::Info {
    let preferred_resolution = get_preferred_resolution(boot_services);
    let mut gop = fetch_gop(boot_services);

    set_resolution(&mut gop, preferred_resolution);

    gop_to_boot_info(&mut gop)
}

fn get_preferred_resolution(bs: &boot::BootServices) -> (u32, u32) {
    let handle = bs
        .get_handle_for_protocol::<edid::DiscoveredProtocol>()
        .unwrap_or_else(|_| todo!());
    stop();

    let edid: ScopedProtocol<'_, edid::DiscoveredProtocol> = bs
        .open_protocol_exclusive(handle)
        .unwrap_or_else(|_| todo!());

    edid.preferred_resolution().unwrap_or_else(|| todo!())
}

fn fetch_gop(boot_services: &boot::BootServices) -> ScopedProtocol<'_, gop::GraphicsOutput> {
    let handle = boot_services
        .get_handle_for_protocol::<gop::GraphicsOutput>()
        .expect("Failed to get handle for GOP protocol!");

    boot_services
        .open_protocol_exclusive(handle)
        .expect("Failed to open GOP protocol!")
}

fn set_resolution(gop: &mut gop::GraphicsOutput, preferred_resolution: (u32, u32)) {
    let (width, height) = preferred_resolution;
    let mode = get_mode_for_preferred_resolution(gop, preferred_resolution);

    gop.set_mode(&mode).expect("Failed to set resolution.");

    info!("width: {} height: {}", width, height);
}

fn gop_to_boot_info(gop: &mut gop::GraphicsOutput) -> vram::Info {
    let resolution: Vec2<usize> = gop.current_mode_info().resolution().into();

    vram::Info::new(
        32,
        resolution.as_(),
        PhysAddr::new(gop.frame_buffer().as_mut_ptr() as u64),
    )
}

fn get_mode_for_preferred_resolution(
    gop: &gop::GraphicsOutput,
    preferred_resolution: (u32, u32),
) -> gop::Mode {
    let (preferred_width, preferred_height) = preferred_resolution;

    for mode in gop.modes() {
        let (width, height) = mode.info().resolution();
        if height == preferred_height.try_into().unwrap_or_else(|_| todo!())
            && width == preferred_width.try_into().unwrap_or_else(|_| todo!())
            && is_usable_gop_mode(mode.info())
        {
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
