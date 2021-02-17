// SPDX-License-Identifier: GPL-3.0-or-later

use core::slice;

use super::{
    endpoint::{Error, NonDefault},
    endpoints_initializer::EndpointsInitializer,
};
use crate::device::pci::xhci::structures::descriptor::Descriptor;
use alloc::vec::Vec;
use page_box::PageBox;
use xhci::context::EndpointType;

pub(super) struct FullyOperational {
    descriptors: Vec<Descriptor>,
    eps: Vec<NonDefault>,
}
impl FullyOperational {
    pub(super) fn new(i: EndpointsInitializer) -> Self {
        let descriptors = i.descriptors();
        let eps = i.endpoints();

        debug!("Endpoints collected");

        Self { eps, descriptors }
    }

    pub(super) fn ty(&self) -> (u8, u8, u8) {
        for d in &self.descriptors {
            if let Descriptor::Interface(i) = d {
                return i.ty();
            }
        }

        unreachable!("HID class must have at least one interface descriptor");
    }

    pub(super) async fn issue_normal_trb<T>(
        &mut self,
        b: &PageBox<T>,
        ty: EndpointType,
    ) -> Result<(), Error> {
        for ep in &mut self.eps {
            if ep.ty() == ty {
                ep.issue_normal_trb(b).await;
                return Ok(());
            }
        }

        Err(Error::NoSuchEndpoint(ty))
    }
}
impl<'a> IntoIterator for &'a mut FullyOperational {
    type Item = &'a mut NonDefault;
    type IntoIter = slice::IterMut<'a, NonDefault>;

    fn into_iter(self) -> Self::IntoIter {
        self.eps.iter_mut()
    }
}
