use std::fmt::{Display, Formatter};
use std::io::ErrorKind;
use std::path::PathBuf;

use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;

pub(crate) struct Product {
    pub(crate) name: String,
    pub(crate) ver: String,
    pub(crate) lang: String,
    pub(crate) install_path: PathBuf,
    pub(crate) exe_names: &'static [&'static str],
}

impl Product {
    fn new(name: String, ver: String, lang: String, install_path: String) -> Result<Self, String> {
        let exe_names: &'static [&'static str] =
            if name.contains("ZennoPoster") && name.contains("V7") {
                &["ProjectMaker", "ZennoPoster"]
            } else if name.contains("ZennoDroid") {
                &["ProjectMakerZD", "ZennoDroid"]
            } else if name.contains("ZennoBox") && name.contains("V7") {
                &["ZennoBox"]
            } else {
                return Err(format!("Unsupported product: '{} {} {}'", name, ver, lang));
            };

        Ok(Self {
            name,
            ver,
            lang,
            install_path: PathBuf::from(install_path),
            exe_names,
        })
    }
}

impl Display for Product {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.name, self.ver, self.lang)
    }
}

pub(crate) fn products_searcher() -> Result<Vec<Product>, String> {
    println!("Начат поиск поддерживаемых продуктов...\n");
    
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let zl_root = hkcu.open_subkey(r"Software\ZennoLab").map_err(|e| {
        format!(r"Не удалось открыть 'HKEY_CURRENT_USER\Software\ZennoLab'. Инфо: '{e}'")
    })?;
    
    let mut products = Vec::<Product>::with_capacity(10);
    
    const KNOWN_LANGS: &[&str] = &["RU", "EN", "CN"];
    
    for lang in KNOWN_LANGS {
        let lang_key = match zl_root.open_subkey(lang) {
            Ok(r) => r,
            Err(e) => {
                if e.kind() != ErrorKind::NotFound {
                    println!(r"Не удалось открыть раздел языка: '{lang}'. Инфо: '{e}'");
                }

                continue;
            }
        };

        for prod_name_res in lang_key.enum_keys() {
            let prod_name = &match prod_name_res {
                Ok(r) => r,
                Err(e) => {
                    println!(
                        r"Ошибка при разборе имени раздела продукта в разделе языка: '{lang}'. Инфо: '{e}'"
                    );
                    continue;
                }
            };
    
            if prod_name.contains("V7") || prod_name.contains("ZennoDroid") {
                let prod_key = match lang_key.open_subkey(prod_name.clone()) {
                    Ok(r) => r,
                    Err(e) => {
                        println!(
                            r"Не удалось открыть раздел продукта: '{prod_name}' языка: '{lang}'. Инфо: '{e}'"
                        );
                        continue;
                    }
                };
        
                for ver_res in prod_key.enum_keys() {
                    let ver = match ver_res {
                        Ok(r) => r,
                        Err(e) => {
                            println!(
                                r"Ошибка при разборе имени раздела версии в разделе продукта: '{prod_name}' языка: '{lang}'. Инфо: '{e}'"
                            );
                            continue;
                        }
                    };
            
                    let ver_key = match prod_key.open_subkey(ver.clone()) {
                        Ok(r) => r,
                        Err(e) => {
                            println!(
                                r"Не удалось открыть раздел продукта: '{prod_name} {ver} {lang}'. Инфо: {e}"
                            );
                            continue;
                        }
                    };
            
                    match ver_key.get_value::<String, _>("SuccessInstall") {
                        Ok(r) => {
                            if r != *"True" {
                                println!(
                                    r"Найден недоустановленный продукт: '{prod_name} {ver} {lang}'"
                                );
                                continue;
                            }
                        }
                        Err(e) => {
                            println!(
                                r"Не удалось получить статус установки продукта: '{prod_name} {ver} {lang}'. Инфо: {e}"
                            );
                            continue;
                        }
                    }
            
                    match ver_key.get_value("InstallDir") {
                        Ok(install_path) => {
                            match Product::new(
                                prod_name.to_string(),
                                ver,
                                lang.to_string(),
                                install_path,
                            ) {
                                Ok(r) => {
                                    println!("Найден продукт: '{}'", &r);
                                    products.push(r)
                                }
                                Err(e) => {
                                    println!("{}", e);
                                    continue;
                                }
                            }
                        }
                        Err(e) => {
                            println!(
                                r"Не удалось получить путь установки продукта: '{prod_name} {ver} {lang}'. Инфо: {e}"
                            );
                            continue;
                        }
                    }
                }
            }
        }
    }
    
    if products.is_empty() {
        return Err(r"Не найден ни один установленный продукт.".to_string());
    }
    
    println!();
    
    Ok(products)
}
