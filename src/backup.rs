use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::ESTIMATED_EXE_CFG_SIZE;
use crate::products_searcher::Product;

pub(crate) fn create_backup(
    mut orig_file: &File,
    orig_name: &str,
    backup_dir: &PathBuf,
    product: &Product,
) -> Result<(), String> {
    let backup_name = format!(
        "{}.{} {} {}.bak",
        orig_name, product.name, product.ver, product.lang
    );

    let mut backup_file_pathbuf = backup_dir.clone();
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
        .map_err(|e| format!(r"Не удалось считать содержимое файла: '{orig_name}'. Инфо: {e}"))?;
    
    let mut backup_file = File::create(backup_file_path).map_err(|e| {
        format!(r"Не удалось создать/открыть файл бекапа: '{backup_file_path:?}'. Инфо: {e}")
    })?;
    
    backup_file.write_all(orig_data.as_slice()).map_err(|e| {
        format!(
            r"Не удалось записать в файл бекапа: '{:?}'. Инфо: {e}",
            backup_file_path
        )
    })?;
    
    println!("Создан бекап: '{}'", backup_name);
    
    Ok(())
}

fn backup_notify(backup_name: &str) {
    println!("Бекап '{backup_name}' уже существует.");
}
