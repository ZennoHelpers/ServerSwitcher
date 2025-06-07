use std::fs::File;
use std::io::Write;
use std::io::{BufReader, Cursor, Seek};
use std::path::PathBuf;

use quick_xml::events::Event;
use quick_xml::events::attributes::{Attr, Attribute};
use quick_xml::{Reader, Writer};
use url::Url;

use crate::ESTIMATED_EXE_CFG_SIZE;

pub(crate) fn switch_server(
    exe_cfg_file: &mut File,
    exe_cfg_path: &PathBuf,
    server_domain: &str,
) -> Result<(), String> {
    println!(
        "Changing the server in the configuration file: '{}'",
        exe_cfg_path.to_str().unwrap()
    );

    let buf_rdr = BufReader::new(exe_cfg_file);

    let mut reader = Reader::from_reader(buf_rdr);

    let mut new_data = Vec::with_capacity(ESTIMATED_EXE_CFG_SIZE);

    let mut writer = Writer::new(Cursor::new(&mut new_data));

    let mut tmp_buf = Vec::with_capacity(ESTIMATED_EXE_CFG_SIZE);

    loop {
        match reader.read_event_into(&mut tmp_buf) {
            Ok(Event::Empty(e)) if e.name().as_ref() == b"endpoint" => {
                let mut edit_elem = e.clone();

                edit_elem.clear_attributes();

                for attr in e.attributes() {
                    let attr = attr.map_err(|e| {
                        format!(
                            "Error during attribute parsing xml. Event: 'Event::Start'. Info: '{}'",
                            e
                        )
                    })?;

                    if attr.key.local_name().as_ref() == b"address" {
                        let url = std::str::from_utf8(attr.value.as_ref())
                            .map_err(|e| format!("Failed to convert the received url to a valid UTF-8 string. Info: '{e}'"))?;
                        let mut url = Url::parse(url).map_err(|e| {
                            format!("Failed to parse the received url. Info: '{e}'")
                        })?;
                        url.set_host(Some(server_domain)).map_err(|e| {
                            format!("Failed to set host to the received url. Info: '{e}'")
                        })?;

                        let new_url_attr = Attribute::from(Attr::DoubleQ(
                            b"address" as &[u8],
                            url.as_str().as_bytes(),
                        ));
                        edit_elem.push_attribute(new_url_attr);
                    } else {
                        edit_elem.push_attribute(attr);
                    }
                }

                writer.write_event(Event::Empty(edit_elem)).map_err(|e| {
                    format!("Writing xml error. Event: 'Event::Start'. Info: '{}'", e)
                })?;
            }
            Ok(Event::Eof) => break,
            Ok(e) => writer
                .write_event(e)
                .map_err(|e| format!("Writing xml error. Info: '{}'", e))?,
            Err(e) => panic!(
                "Reading xml error. Error at position {}: {e:?}",
                reader.buffer_position()
            ),
        }
    }

    let bs = String::from_utf8(new_data).map_err(|e| {
        format!(
            r"Failed to convert the created xml to a string. Info: '{}'",
            e
        )
    })?;

    let exe_cfg_file = reader.get_mut().get_mut(); // получаем захваченный File обратно из нёдр...
    exe_cfg_file
        .rewind()
        .map_err(|e| format!("File rewind '{exe_cfg_path:?}' failed. Info: '{e}'"))?;
    exe_cfg_file
        .set_len(0)
        .map_err(|e| format!("File emptying '{exe_cfg_path:?}' failed. Info: '{e}'"))?;
    exe_cfg_file
        .write_all(bs.as_bytes())
        .map(|_| {
            println!(
                "Configuration file: '{}' has been modified.\n",
                exe_cfg_path.to_str().unwrap()
            )
        })
        .unwrap();

    Ok(())
}
