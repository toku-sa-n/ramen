use {
    crate::process::ipc,
    core::{
        convert::{TryFrom, TryInto},
        mem::MaybeUninit,
    },
    log::warn,
    message::Message,
    num_traits::FromPrimitive,
    x86_64::{
        instructions::port::{PortReadOnly, PortWriteOnly},
        structures::port::{PortRead, PortWrite},
        VirtAddr,
    },
};

pub(crate) fn main() -> ! {
    main_loop();
}

fn main_loop() -> ! {
    loop {
        main_loop_iteration();
    }
}

fn main_loop_iteration() {
    let m = MaybeUninit::uninit();

    ipc::receive_from_any(VirtAddr::from_ptr(m.as_ptr()));
    handle_message(unsafe { m.assume_init() });
}

fn handle_message(m: Message) {
    let t = FromPrimitive::from_u64(m.body.0);
    if let Some(t) = t {
        select_system_calls(m, t);
    } else {
        warn!("Unrecognized message: {:?}", m);
    }
}

fn select_system_calls(m: Message, t: syscalls::Ty) {
    match t {
        syscalls::Ty::Inb => unsafe { reply_inb(m) },
        syscalls::Ty::Inl => unsafe { reply_inl(m) },
        syscalls::Ty::Outb => unsafe { reply_outb(m) },
        syscalls::Ty::Outl => unsafe { reply_outl(m) },
        _ => panic!("Not supported: {:?}", t),
    }
}

unsafe fn reply_inb(m: Message) {
    // SAFETY: The caller must ensure that the message contains the correct values.
    let r = unsafe { inb(m) };
    reply_with_result(m, r.into());
}

unsafe fn reply_inl(m: Message) {
    // SAFETY: The caller must ensure that the message contains the correct values.
    let r = unsafe { inl(m) };
    reply_with_result(m, r.into());
}

unsafe fn reply_outb(m: Message) {
    // SAFETY: The caller must ensure that the message contains the correct values.
    unsafe { outb(m) };
    reply_without_contents(m);
}

unsafe fn reply_outl(m: Message) {
    // SAFETY: The caller must ensure that the message contains the correct values.
    unsafe { outl(m) };
    reply_without_contents(m);
}

fn reply_with_result(received: Message, result: u64) {
    let h = message::Header::default();
    let b = message::Body(result, 0, 0, 0, 0);

    let reply = Message::new(h, b);
    let to = received.header.sender;

    ipc::send(VirtAddr::from_ptr(&reply), to);
}

fn reply_without_contents(received: Message) {
    let h = message::Header::default();
    let b = message::Body::default();

    let reply = Message::new(h, b);
    let to = received.header.sender;

    ipc::send(VirtAddr::from_ptr(&reply), to);
}

pub(super) unsafe fn inb(m: Message) -> u8 {
    unsafe { read_from_port(m) }
}

pub(super) unsafe fn inl(m: Message) -> u32 {
    unsafe { read_from_port(m) }
}

pub(super) unsafe fn outb(m: Message) {
    unsafe {
        write_to_port::<u8>(m);
    }
}

pub(super) unsafe fn outl(m: Message) {
    unsafe {
        write_to_port::<u32>(m);
    }
}

unsafe fn read_from_port<T: PortRead>(m: Message) -> T {
    let p = m.body.1;
    let mut p = PortReadOnly::new(p.try_into().unwrap());

    unsafe { p.read() }
}

unsafe fn write_to_port<T: PortWrite + TryFrom<u64>>(m: Message)
where
    <T as TryFrom<u64>>::Error: core::fmt::Debug,
{
    let message::Body(_, p, v, ..) = m.body;
    let mut p = PortWriteOnly::<T>::new(p.try_into().unwrap());

    unsafe {
        p.write(T::try_from(v).unwrap());
    }
}
