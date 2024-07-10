use std::fs::File;
use std::io::{Seek, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, panic};

use crate::backup::create_backup;
use crate::products_searcher::products_searcher;
use crate::server_selection::server_selection;
use crate::server_switcher::switch_server;

mod backup;
mod products_searcher;
mod server_selection;
mod server_switcher;

const ESTIMATED_EXE_CFG_SIZE: usize = 60000;

fn start() -> Result<(), String> {
    println!("Current language: Russian (only it's supported).\n");

    let products = products_searcher()?;

    let backup_dir = env::current_dir()
        .map_err(|e| format!(r"Не удалось получить рабочую директорию. Инфо: {e}"))?;

    let server_index = env::args().enumerate().find_map(|(i, v)| {
        if i == 2 {
            Some(
                usize::from_str(&v)
                    .map_err(|e| format!("Не удалось распарсить аргумент запуска. Инфо: '{e}'"))
                    .unwrap(),
            )
        } else {
            None
        }
    });

    let server = server_selection(server_index)?;

    println!("Создание бекапов и изменение домена сервера...\n");
    for product in products {
        for exe_name in product.exe_names {
            let cfg_fullname = format!(r"{exe_name}.exe.config");

            let exe_cfg_path = get_exe_config_path(&product.install_path, &cfg_fullname);

            match File::options().read(true).write(true).open(&exe_cfg_path) {
                Ok(mut exe_cfg_file) => {
                    create_backup(&exe_cfg_file, &cfg_fullname, &backup_dir, &product)?;

                    // Сброс курсора в начало файла
                    exe_cfg_file.rewind().map_err(|e| {
                        format!("Rewind файла '{exe_cfg_path:?}' завершился ошибкой. Инфо: '{e}'")
                    })?;

                    switch_server(&mut exe_cfg_file, &exe_cfg_path, server)?;
                }
                Err(e) => {
                    println!(r"Не удалось открыть файл настроек PM: '{exe_cfg_path:?}'. Инфо: {e}")
                }
            }
        }
    }

    if server_index.is_none() {
        println!("\nРабота завершена.");
        pause();
    }

    Ok(())
}

fn main() {
    if let Err(e) = panic::catch_unwind(|| {
        if let Err(e) = start() {
            println!("{}", e);
            pause();
        }
    }) {
        println!("Перехвачена паника. Инфо:\n{:?}", e.as_ref());
        pause();
    }
}

fn get_exe_config_path(path: &Path, cfg_fullname: &str) -> PathBuf {
    let mut cfg_path = PathBuf::from(path);
    cfg_path.push("Progs");
    cfg_path.push(cfg_fullname);
    cfg_path
}

fn pause() {
    println!("Нажмите Enter для завершения.");
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut String::new()).unwrap();
}
