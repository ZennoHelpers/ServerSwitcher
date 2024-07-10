use std::fmt::{Display, Formatter};
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
        let exe_names: &'static [&'static str] = get_exe_names(&name, &ver, &lang)?;

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

fn get_exe_names(name: &str, ver: &str, lang: &str) -> Result<&'static [&'static str], String> {
    Ok(if name.contains("ZennoPoster") && name.contains("V7") {
        &["ProjectMaker", "ZennoPoster"]
    } else if name.contains("ZennoProjectMaker") {
        &["ProjectMaker", "ProjectMakerZD"]
    } else if name.contains("ZennoDroid") {
        &["ProjectMakerZD", "ZennoDroid"]
    } else if name.contains("ZennoBox") && name.contains("V7") {
        &["ZennoBox"]
    } else if name.contains("ProxyChecker") {
        &["ProxyChecker"]
    } else if name.contains("CapMonster") {
        &["CapMonster", "CapMonsterMCS", "LicenseHelper"]
    } else {
        return Err(format!("Unsupported product: '{} {} {}'", name, ver, lang));
    })
}

pub(crate) fn products_searcher() -> Result<Vec<Product>, String> {
    println!("Начат поиск поддерживаемых продуктов...\n");

    let mut products = Vec::<Product>::with_capacity(10);

    RegKey::predef(HKEY_CURRENT_USER).open_subkey(r"Software\ZennoLab").consume(|zl_root| {
        zl_root.enum_keys().for_each(| key | {
            key.consume(| key | {
                if key.len() == 2 && key == key.to_uppercase() {
                    let lang = key;

                    zl_root.open_subkey(lang.clone()).consume(|lang_key| {
                        lang_key.enum_keys().for_each(|prod_name_res| {
                            prod_name_res.consume(|prod_name| {
                                lang_key.open_subkey(&prod_name).consume(|prod_key| {
                                    prod_key.enum_keys().for_each(|ver_res| {
                                        ver_res.consume(|ver| {
                                            prod_key.open_subkey(&ver).consume(|ver_key| {
                                                ver_key.get_value::<String, _>("SuccessInstall").consume(|install| {
                                                        if install != "True" {
                                                            println!(r"Найден недоустановленный продукт: '{prod_name} {ver} {lang}'");
                                                        }
                                                    }, |e| {
                                                        println!(r"Не удалось получить статус установки продукта: '{prod_name} {ver} {lang}'. Инфо: {e}");
                                                    });

                                                ver_key.get_value("InstallDir").consume(|install_path: String| {
                                                    Product::new(
                                                        prod_name.clone(),
                                                        ver.clone(),
                                                        lang.to_owned(),
                                                        install_path,
                                                    ).consume(|product| {
                                                            println!("Найден продукт: '{}'", &product);
                                                            products.push(product)
                                                        }, |e| {
                                                            println!("{}", e);
                                                        });
                                                }, |e| {
                                                    println!(
                                                        r"Не удалось получить путь установки продукта: '{prod_name} {ver} {lang}'. Инфо: {e}"
                                                    );
                                                });
                                            }, |e| {
                                                println!(
                                                    r"Не удалось открыть раздел продукта: '{prod_name} {ver} {lang}'. Инфо: {e}"
                                                );
                                            });
                                        }, |e| {
                                            println!(
                                                r"Ошибка при разборе имени раздела версии в разделе продукта: '{prod_name}' языка: '{lang}'. Инфо: '{e}'"
                                            );
                                        });
                                    });
                                }, |e| {
                                    println!(
                                        r"Не удалось открыть раздел продукта: '{prod_name}' языка: '{lang}'. Инфо: '{e}'"
                                    );
                                });
                            }, |e| {
                                println!(
                                    r"Ошибка при разборе имени раздела продукта в разделе языка: '{lang}'. Инфо: '{e}'"
                                );
                            });
                        });
                    }, |e|{
                        println!(r"Не удалось открыть раздел языка: '{lang}'. Инфо: '{e}'");
                    });
                }
            }, |e|{
                println!(r"Не удалось получить раздел реестра в 'HKEY_CURRENT_USER\Software\ZennoLab'. Инфо: '{e}'");
            });
        });

        }, |e| {
            println!(r"Не удалось открыть 'HKEY_CURRENT_USER\Software\ZennoLab'. Инфо: '{e}'")
        });

    if products.is_empty() {
        return Err(r"Не найден ни один установленный продукт.".to_string());
    }

    println!();

    Ok(products)
}

trait ResultConsumerTrait<R, E> {
    fn consume<F1: FnOnce(R), F2: FnOnce(E)>(self, f1: F1, f2: F2);
}

impl<R, E> ResultConsumerTrait<R, E> for Result<R, E> {
    #[inline]
    fn consume<F1: FnOnce(R), F2: FnOnce(E)>(self, f1: F1, f2: F2) {
        match self {
            Ok(r) => f1(r),
            Err(e) => f2(e),
        }
    }
}
