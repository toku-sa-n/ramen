// SPDX-License-Identifier: GPL-3.0-or-later

use {
    alloc::boxed::Box,
    core::{
        future::Future,
        pin::Pin,
        task::{Context, Poll},
    },
};

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Self {
            future: Box::pin(future),
        }
    }

    pub fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
