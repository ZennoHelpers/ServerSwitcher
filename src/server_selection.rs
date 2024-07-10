use std::io;
use std::io::BufRead;

pub(crate) fn server_selection(server_index: Option<usize>) -> Result<&'static str, String> {
    let servers_list = [
        "userarea.zenno.io",
        "userarea.zennolab.com",
        "userarea-us.zennolab.com",
    ];
    
    if let Some(r) = server_index {
        if r == 0 || r > servers_list.len() {
            return Err(format!("Недопустимый индекс сервера: '{r}'"))
        }
        return Ok(servers_list[r - 1])
    }
    
    for x in servers_list.iter().enumerate() {
        println!("{}: '{}'", x.0 + 1, x.1)
    }
    
    println!("\nУкажите индекс сервера и нажмите Enter:");
    
    let buffer = &mut String::with_capacity(3);
    for _ in 1..=3 {
        let mut stdin = io::stdin().lock();
        stdin
            .read_line(buffer)
            .map_err(|e| format!("Ошибка при считывании ввода. Инфо: '{e}'"))?;
        let index = match buffer.chars().next().unwrap() {
            '1' => 0,
            '2' => 1,
            '3' => 2,
            _ => {
                println!("Неверный ввод.'");
                buffer.clear();
                continue;
            }
        };
        
        if let Some(r) = servers_list.get(index) {
            println!("\nВыбран: '{}'\n", r);
            return Ok(r);
        }
    }
    Err("Превышенно кол-во попыток ввода.".to_string())
}
