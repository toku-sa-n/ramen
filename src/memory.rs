struct Type(u32);
struct Attr(u64);

const LOADER_DATA: Type = Type(1);

pub struct MapInfo {
    ptr: *mut MapDescriptor,
    count: usize,
}

pub struct MapDescriptor {
    ty: Type,
    phys: u64,
    virt: u64,
    num_pages: u64,
    attrs: Attr,
}
