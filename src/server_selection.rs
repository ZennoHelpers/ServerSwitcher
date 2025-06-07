use std::io;
use std::io::BufRead;

pub(crate) fn server_selection(server_index: Option<usize>) -> Result<&'static str, String> {
    let servers_list: &[&'static str] = &[
        "userarea.zenno.io",
        "userarea.zennolab.com",
        "userarea-us.zennolab.com",
        "userarea-hk.zennolab.com",
    ];

    if let Some(r) = server_index {
        if r == 0 || r > servers_list.len() {
            return Err(format!("Invalid server number: '{r}'"));
        }
        return Ok(servers_list[r - 1]);
    }

    for x in servers_list.iter().enumerate() {
        println!("{}: '{}'", x.0 + 1, x.1)
    }

    println!("\nType server number and press Enter:");

    let buffer = &mut String::with_capacity(3);
    for _ in 1..=3 {
        let mut stdin = io::stdin().lock();

        stdin
            .read_line(buffer)
            .map_err(|e| format!("Error when parse input. Info: '{e}'"))?;

        if let Some(input) = buffer.chars().next() {
            if let Some(r) = input.to_digit(10) {
                let number = r - 1;

                if let Some(r) = servers_list.get(number as usize) {
                    println!("\nSelected: '{}'\n", r);
                    return Ok(r);
                } else {
                    warn(buffer);
                    continue;
                }
            } else {
                warn(buffer);
                continue;
            }
        } else {
            warn(buffer);
            continue;
        }
    }
    Err("Exceeded number of input attempts.".to_string())
}

fn warn(buffer: &mut String) {
    println!("Incorrect input.");
    buffer.clear();
}
