// SPDX-License-Identifier: GPL-3.0-or-later

use common::{kernelboot, vram};
use conquer_once::spin::OnceCell;
use message::Message;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

static VRAM_INFO: OnceCell<vram::Info> = OnceCell::uninit();

pub(super) fn init(i: &kernelboot::Info) {
    save_vram_info(i)
}

pub(super) fn main() {
    loop {
        loop_iteration()
    }
}

fn save_vram_info(i: &kernelboot::Info) {
    VRAM_INFO.init_once(|| i.vram())
}

fn loop_iteration() {
    let m = syscalls::receive_from_any();
    handle_message(&m);
}

fn handle_message(m: &Message) {
    if let Some(Ty::GetVramInfo) = FromPrimitive::from_u64(m.body.0) {
        send_vram_info(m.header.sender);
    } else {
        panic!("Unrecognized message: {:?}", m)
    }
}

fn send_vram_info(sender: i32) {
    let i = VRAM_INFO.try_get();
    let i = i.expect("Failed to get the kernel boot information.");

    let h = message::Header::default();
    let b = message::Body(
        i.resolution().x.into(),
        i.resolution().y.into(),
        i.bpp().into(),
        0,
        0,
    );
    let r = Message::new(h, b);

    syscalls::send(r, sender);
}

#[derive(FromPrimitive)]
enum Ty {
    GetVramInfo,
}
