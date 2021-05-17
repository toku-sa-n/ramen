use os_units::NumOfPages;
use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame, Size4KiB};

pub(crate) struct Allocator;
unsafe impl FrameAllocator<Size4KiB> for Allocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let a = syscalls::allocate_phys_frame(NumOfPages::new(1));
        let f = PhysFrame::from_start_address(a);
        let f = f.expect("Physical frame is not page-aligned.");

        Some(f)
    }
}
impl FrameDeallocator<Size4KiB> for Allocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        let a = frame.start_address();

        syscalls::deallocate_phys_frame(a, NumOfPages::new(1));
    }
}
