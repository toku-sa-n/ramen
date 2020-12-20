// SPDX-License-Identifier: GPL-3.0-or-later

struct Context {
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rdi: u64,
    rsi: u64,
    rsp: u64,
    rbp: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,

    cr3: u64,
    rip: u64,
    rflags: u64,

    es: u64,
    cs: u64,
    ss: u64,
    ds: u64,
    fs: u64,
    gs: u64,
}
