use crate::page_table;
use alloc::collections::BTreeMap;
use x86_64::PhysAddr;

struct Collection {
    pml4: page_table::Boxed,
    pdpt: BTreeMap<PhysAddr, page_table::Boxed>,
    dir: BTreeMap<PhysAddr, page_table::Boxed>,
    table: BTreeMap<PhysAddr, page_table::Boxed>,
}
