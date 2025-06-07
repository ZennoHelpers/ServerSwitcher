use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use zennolab_products_searcher::ZennoLabProduct;

use crate::ESTIMATED_EXE_CFG_SIZE;

pub(crate) fn create_backup<'a>(
    mut orig_file: &'a File,
    orig_name: &'a str,
    backup_dir: &'a Path,
    p: &'a ZennoLabProduct,
) -> Result<(), String> {
    let backup_name = format!(
        "{}.{} {} {}.bak",
        orig_name, p.name(), p.ver(), p.lang()
    );

    let mut backup_file_pathbuf = PathBuf::from(backup_dir);
    backup_file_pathbuf.push(&backup_name);

    let backup_file_path = Path::new(&backup_file_pathbuf);

    if backup_file_path.exists() {
        // TODO переделать в запрос на перезапись
        backup_notify(&backup_name);
        return Ok(());
    }

    let mut orig_data = Vec::<u8>::with_capacity(ESTIMATED_EXE_CFG_SIZE);

    orig_file
        .read_to_end(&mut orig_data)
        .map_err(|e| format!(r"Failed to read file contents: '{orig_name}'. Info: {e}"))?;

    let mut backup_file = File::create(backup_file_path).map_err(|e| {
        format!(r"Failed to create/open backup file: '{backup_file_path:?}'. Info: {e}")
    })?;

    backup_file.write_all(orig_data.as_slice()).map_err(|e| {
        format!(
            r"Failed to write backup file: '{:?}'. Info: {e}",
            backup_file_path
        )
    })?;

    println!("Backup created: '{}'", backup_name);

    Ok(())
}

fn backup_notify(backup_name: &str) {
    println!("Backup '{backup_name}' already exists.");
}
