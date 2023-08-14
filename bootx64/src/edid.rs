use {core::slice, uefi::proto::unsafe_protocol};

#[unsafe_protocol("1c0c34f6-d380-41fa-a049-8ad06c1a66aa")]
pub(crate) struct DiscoveredProtocol {
    size: u32,
    ptr: *const u8,
}
impl DiscoveredProtocol {
    #[must_use]
    pub(crate) fn preferred_resolution(&self) -> Option<(u32, u32)> {
        Some((self.preferred_width()?, self.preferred_height()?))
    }

    fn preferred_width(&self) -> Option<u32> {
        let info = self.get_info()?;

        let upper = (u32::from(info[58]) & 0xf0) << 4;
        let lower: u32 = info[56].into();

        Some(upper | lower)
    }

    fn preferred_height(&self) -> Option<u32> {
        let info = self.get_info()?;

        let upper = (u32::from(info[61]) & 0xf0) << 4;
        let lower: u32 = info[59].into();

        Some(upper | lower)
    }

    fn get_info(&self) -> Option<&[u8]> {
        self.info_exists()
            .then(|| unsafe { self.get_info_unchecked() })
    }

    unsafe fn get_info_unchecked(&self) -> &[u8] {
        let sz: usize = self.size.try_into().unwrap();

        // SAFETY: `self.ptr` is valid for `sz` bytes as it is not null. These memory are not
        // modified.
        unsafe { slice::from_raw_parts(self.ptr, sz) }
    }

    fn info_exists(&self) -> bool {
        !self.ptr.is_null()
    }
}
