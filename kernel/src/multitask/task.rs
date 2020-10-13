// SPDX-License-Identifier: GPL-3.0-or-later

use {
    alloc::boxed::Box,
    core::{
        future::Future,
        pin::Pin,
        sync::atomic::{AtomicU64, Ordering},
        task::{Context, Poll},
    },
};

#[derive(PartialOrd, PartialEq, Ord, Eq, Copy, Clone, Debug)]
pub(super) struct Id(u64);

impl Id {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        Id(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct Task {
    id: Id,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Self {
            id: Id::new(),
            future: Box::pin(future),
        }
    }

    pub fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }

    pub(super) fn id(&self) -> Id {
        self.id
    }
}
