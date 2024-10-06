use serde::{ Deserialize, Serialize };
use willhook::{ willhook, IsSystemKeyPress, KeyPress, KeyboardKey };
use std::{
    collections::HashSet,
    fmt::Debug,
    fs::{ self },
    io::Write,
    sync::{ atomic::{ AtomicBool, Ordering }, Arc },
};

#[derive(Serialize, Deserialize, Debug)]
struct KeyConfig {
    keys: Vec<KeyInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
struct KeyInfo {
    key_code: Option<KeyboardKey>,
    presses: i32,
    pressed: u8,
}

fn main() {
    let data = fs::read_to_string("./keys.json").expect("Unable to read file");
    let mut res: KeyConfig = serde_json::from_str(&data).expect("Unable to parse");

    let mut enabled_keys: HashSet<Option<KeyboardKey>> = HashSet::new();

    for key in &res.keys {
        enabled_keys.insert(key.key_code);
    }

    println!("{:?}", enabled_keys);

    let is_running = Arc::new(AtomicBool::new(true));
    let set_running = is_running.clone();

    let h = willhook().unwrap();
    let mut keys_pressed: HashSet<Option<KeyboardKey>> = HashSet::new();

    ctrlc
        ::set_handler(move || {
            set_running.store(false, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");

    while is_running.load(Ordering::SeqCst) {
        if let Ok(ie) = h.try_recv() {
            match ie {
                willhook::InputEvent::Keyboard(ke) => {
                    if
                        ke.pressed == KeyPress::Down(IsSystemKeyPress::Normal) &&
                        !keys_pressed.contains(&ke.key) &&
                        enabled_keys.contains(&ke.key)
                    {
                        keys_pressed.insert(ke.key);

                        // stuff //////////////////////////////////////////////////////////////////////
                        for key in &mut res.keys {
                            if key.key_code == ke.key {
                                key.presses += 1;
                                key.pressed += 1;
                            }
                        }

                        let updated_json = serde_json::to_string_pretty(&res).expect("Error");

                        let mut f = std::fs::OpenOptions
                            ::new()
                            .write(true)
                            .open("./keys.json")
                            .expect("Error opening file");
                        f.write_all(&updated_json.as_bytes()).unwrap();
                        let _ = f.flush();
                        /////////////////////////////////////////////////////////////////////////

                        println!("{:?}", keys_pressed);
                    } else if
                        ke.pressed == KeyPress::Up(IsSystemKeyPress::Normal) &&
                        keys_pressed.contains(&ke.key) &&
                        enabled_keys.contains(&ke.key)
                    {
                        keys_pressed.take(&ke.key);

                        // stuff //////////////////////////////////////////////////////////////////////
                        for key in &mut res.keys {
                            if key.key_code == ke.key {
                                key.pressed -= 1;
                            }
                        }

                        let updated_json = serde_json::to_string_pretty(&res).expect("Error");

                        let mut f = std::fs::OpenOptions
                            ::new()
                            .write(true)
                            .open("./keys.json")
                            .expect("Error opening file");
                        f.write_all(&updated_json.as_bytes()).unwrap();
                        let _ = f.flush();
                        /////////////////////////////////////////////////////////////////////////
                    }
                }
                _ => (),
            }
        } else {
            std::thread::yield_now();
        }
    }
}
