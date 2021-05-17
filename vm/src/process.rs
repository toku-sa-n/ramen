use crate::page_table;

struct Process {
    page_tables: page_table::Collection,
}
