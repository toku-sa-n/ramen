use common_items::size::{Byte, Size};
use uefi::proto::media::file;
use uefi::proto::media::file::RegularFile;

pub fn get(root_dir: &mut file::Directory) -> Size<Byte> {
    let mut handler = super::get_handler(root_dir);

    handler
        .set_position(RegularFile::END_OF_FILE)
        .expect("Failed to calculate the size of the kernel.")
        .unwrap();

    Size::new(
        handler
            .get_position()
            .expect("Failed to calculate the size of the kernel.")
            .unwrap() as _,
    )
}
