use crate::engima_machine::{encode, Rotor};
use crate::AppState;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::EventPump;
use std::collections::HashMap;
use std::io::Write;

fn dist(x1: i32, y1: i32, x2: i32, y2: i32) -> f64 {
    (((x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2)) as f64).sqrt()
}

pub fn encode_and_output(k: Keycode, plugboard: &HashMap<u8, u8>, rotors: &mut [Rotor; 3]) {
    if k.to_string().len() != 1 {
        return;
    }

    let key_ch = k.to_string().as_bytes()[0].to_ascii_lowercase();
    if key_ch.is_ascii_lowercase() {
        let encoded = encode(key_ch, plugboard, rotors);
        print!("{}", encoded);
        ::std::io::stdout()
            .flush()
            .map_err(|e| e.to_string())
            .expect("Failed to flush stdout!");
    }
}

pub fn update_rotors(
    event_pump: &EventPump,
    rotors: &mut [Rotor; 3],
    rotor_start_pos: &mut [u8; 3],
    can_click: &mut bool,
) {
    for i in 0..rotors.len() {
        let x = 620 + i as i32 * 32;
        let y = 48;

        let mouse_x = event_pump.mouse_state().x();
        let mouse_y = event_pump.mouse_state().y();

        if event_pump
            .mouse_state()
            .is_mouse_button_pressed(MouseButton::Left)
            && *can_click
            && dist(x + 16, y + 16, mouse_x, mouse_y) < 16.0
        {
            rotors[i].current_value += 1;
            rotors[i].current_value %= 26;
            rotor_start_pos[i] = rotors[i].current_value;
            *can_click = false;
        }
    }
}

pub fn handle_keyboard_click(
    event_pump: &EventPump,
    can_click: &mut bool,
    plugboard: &HashMap<u8, u8>,
    rotors: &mut [Rotor; 3],
) {
    for ch in b'a'..(b'z' + 1_u8) {
        let offset = (ch - b'a') as i32;
        let x = 16 + (offset % 8) * 72;
        let y = 48 + (offset / 8) * 64;

        let mouse_x = event_pump.mouse_state().x();
        let mouse_y = event_pump.mouse_state().y();

        if dist(x + 16, y + 16, mouse_x, mouse_y) < 16.0
            && event_pump
                .mouse_state()
                .is_mouse_button_pressed(MouseButton::Left)
            && *can_click
        {
            let encoded = encode(ch, plugboard, rotors);
            print!("{encoded}");
            ::std::io::stdout()
                .flush()
                .map_err(|e| e.to_string())
                .expect("Failed to flush stdout!");

            *can_click = false;
        }
    }
}

fn key_equal(map: &HashMap<u8, u8>, ch: u8) -> bool {
    if let Some(c2) = map.get(&ch) {
        if *c2 == ch {
            return true;
        }
    }

    false
}

pub fn handle_plugboard_click(
    event_pump: &EventPump,
    plugboard: &mut HashMap<u8, u8>,
    selected_plugboard: Option<u8>,
    can_click: &mut bool,
) -> Option<u8> {
    for ch in b'a'..(b'z' + 1_u8) {
        let offset = (ch - b'a') as i32;
        let x = 16 + (offset % 8) * 72;
        let y = 348 + (offset / 8) * 64;

        let mouse_x = event_pump.mouse_state().x();
        let mouse_y = event_pump.mouse_state().y();

        if !(dist(x + 16, y + 16, mouse_x, mouse_y) < 16.0
            && event_pump
                .mouse_state()
                .is_mouse_button_pressed(MouseButton::Left)
            && *can_click)
        {
            continue;
        }

        *can_click = false;
        let has_plug = key_equal(plugboard, ch);

        match selected_plugboard {
            Some(c) => {
                if c != ch && has_plug {
                    plugboard.insert(c, ch);
                    plugboard.insert(ch, c);
                    return None;
                }
            }
            _ => {
                if has_plug {
                    return Some(ch);
                } else if let Some(c2) = plugboard.get(&ch) {
                    plugboard.insert(*c2, *c2);
                    plugboard.insert(ch, ch);
                }
            }
        }
    }

    selected_plugboard
}

fn reset_rotors(rotors: &mut [Rotor; 3], rotor_start_pos: &[u8; 3]) {
    for (i, rotor) in rotors.iter_mut().enumerate() {
        rotor.current_value = rotor_start_pos[i];
    }
    println!();
}

pub fn process_events(event_pump: &mut EventPump, state: &mut AppState) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => state.can_quit = true,
            Event::MouseButtonUp {
                mouse_btn: MouseButton::Left,
                ..
            } => state.can_click = true,
            //Reset rotors
            Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => reset_rotors(&mut state.rotors, &state.rotor_start_pos),
            Event::KeyDown {
                keycode: Some(k), ..
            } => encode_and_output(k, &state.plugboard, &mut state.rotors),
            _ => {}
        }
    }
}
