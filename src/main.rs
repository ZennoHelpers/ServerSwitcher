use std::env;
use std::fs::File;
use std::io::{Seek, Write};
use std::path::PathBuf;
use std::str::FromStr;

use crate::backup::create_backup;
use crate::products_searcher::products_searcher;
use crate::server_changer::change_server;
use crate::server_selection::server_selection;

mod backup;
mod products_searcher;
mod server_changer;
mod server_selection;

const ESTIMATED_EXE_CFG_SIZE: usize = 60000;

fn main() -> Result<(), String> {
    println!("Current language: Russian (only it's supported).\n");
    
    let products = products_searcher()?;
    
    let backup_dir = env::current_dir()
        .map_err(|e| format!(r"Не удалось получить рабочую директорию. Инфо: {e}"))?;
    
    let (server, is_manual) = if let Some(r) = env::args().collect::<Vec<_>>().get(1) {
        (server_selection(Some(usize::from_str(r).map_err(|e| {
            format!("Не удалось распарсить аргумент запуска. Инфо: '{e}'")
        })?))?, false)
    } else {
        (server_selection(None)?, true)
    };
    
    println!("Создание бекапов и установка сервера...\n");
    for product in products {
        for exe_name in product.exe_names {
            let cfg_fullname = format!(r"{exe_name}.exe.config");
            
            let exe_cfg_path = get_exe_config_path(&product.install_path, &cfg_fullname);
            
            let mut exe_cfg_file = File::options()
                .read(true)
                .write(true)
                .open(&exe_cfg_path)
                .map_err(|e| {
                    format!(r"Не удалось открыть файл настроек PM: '{exe_cfg_path:?}'. Инфо: {e}")
                })?;
            
            create_backup(&exe_cfg_file, &cfg_fullname, &backup_dir, &product)?;
            
            // Сброс курсора в начало файла
            exe_cfg_file.rewind().map_err(|e| {
                format!("Rewind файла '{exe_cfg_path:?}' завершился ошибкой. Инфо: '{e}'")
            })?;
            
            change_server(&mut exe_cfg_file, &exe_cfg_path, server)?;
        }
    }
    
    if is_manual {
        println!("\nРабота завершена.\nНажмите Enter.");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut String::new()).unwrap();
    }
    
    Ok(())
}

fn get_exe_config_path(path: &PathBuf, cfg_fullname: &str) -> PathBuf {
    let mut cfg_path = path.clone();
    cfg_path.push("Progs");
    cfg_path.push(cfg_fullname);
    cfg_path
}
