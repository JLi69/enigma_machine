use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};
use std::collections::HashMap;
use std::io::Write;

/*
 * Enigma machine encryption:
 * Enter letter
 * Increment rotors
 * Put letter through plugboard
 * Go through rotors
 * Go through rotors in reverse
 * Go through plugboard
 * Output letter
 * */

const REFLECTOR: &str = "yruhqsldpxngokmiebfzcwvjat";

struct Rotor<'a> {
    current_value: u8,
    code_string: &'a str,
    rotate_value: u8,
}

pub fn display_text_left_justify(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    x: i32,
    y: i32,
    font: &Font,
    text: &str,
    col: Color,
    char_sz: u32,
) -> Result<(), String> {
    let font_surface = font.render(text).solid(col).map_err(|e| e.to_string())?;
    let font_texture = texture_creator
        .create_texture_from_surface(&font_surface)
        .map_err(|e| e.to_string())?;
    canvas
        .copy(
            &font_texture,
            None,
            Rect::new(x, y, char_sz * text.len() as u32, char_sz * 2),
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn encode(ch: u8, plugboard: &HashMap<u8, u8>, rotors: &mut [Rotor; 3]) -> char {
    //Increment rotors
    let rotor2_increment = rotors[1].current_value + b'a' == rotors[1].rotate_value
        || rotors[0].current_value + b'a' == rotors[0].rotate_value;
    let rotor3_increment = rotors[1].current_value + b'a' == rotors[1].rotate_value;

    rotors[0].current_value += 1;
    rotors[0].current_value %= 26;
    if rotor2_increment {
        rotors[1].current_value += 1;
        rotors[1].current_value %= 26;
    }
    if rotor3_increment {
        rotors[2].current_value += 1;
        rotors[2].current_value %= 26;
    }

    let mut encoded: u8 = 0;

    //Go through plugboard
    match plugboard.get(&ch) {
        Some(pair) => {
            encoded = *pair;
        }
        _ => {}
    }

    //Go through rotors
    for i in 0..3 {
        let index = ((encoded - b'a' + 26 - rotors[i].current_value) % 26) as usize;
        encoded = rotors[i].code_string.as_bytes()[index];
    }

    encoded = REFLECTOR.as_bytes()[(encoded - b'a') as usize];

    //Go through rotors again
    for i in 0..3 {
        let ind = 2 - i;

        let mut index = 0;
        for j in 0..26 {
            if rotors[ind].code_string.as_bytes()[j] == encoded {
                index = (j + rotors[ind].current_value as usize) % 26;
                break;
            }
        }
        encoded = index as u8 + b'a';
    }

    //Go through plugboard again
    match plugboard.get(&encoded) {
        Some(pair) => {
            encoded = *pair;
        }
        _ => {}
    }

    //Return the encoded character
    encoded as char
}

fn main() -> Result<(), String> {
    let sdl_ctx = sdl2::init().unwrap();
    let video_subsystem = sdl_ctx.video().unwrap();
    let window = video_subsystem
        .window("Enigma Machine", 750, 600)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_ctx.event_pump().unwrap();

    let font_ctx = sdl2::ttf::init().unwrap();
    let font = font_ctx
        .load_font("fonts/8BitOperator/8bitOperatorPlus8-Regular.ttf", 32)
        .unwrap();

    let texture_creator = canvas.texture_creator();

    let mut can_click = true;

    let mut selected_plugboard: Option<u8> = None;

    let mut rotors = [
        //I
        Rotor {
            current_value: 0,
            code_string: "jgdqoxuscamifrvtpnewkblzyh",
            rotate_value: b'q',
        },
        //II
        Rotor {
            current_value: 0,
            code_string: "ajdksiruxblhwtmcqgznpyfvoe",
            rotate_value: b'e',
        },
        //III
        Rotor {
            current_value: 0,
            code_string: "bdfhjlcprtxvznyeiwgakmusqo",
            rotate_value: b'v',
        },
    ];

    let mut plugboard = HashMap::<u8, u8>::new();
    for ch in b'a'..(b'z' + 1 as u8) {
        plugboard.insert(ch, ch);
    }

    'running: loop {
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();

        display_text_left_justify(
            &mut canvas,
            &texture_creator,
            16,
            16,
            &font,
            "Enigma Machine",
            Color::YELLOW,
            8,
        )
        .map_err(|e| e.to_string())?;

        //Display keyboard
        for ch in b'a'..(b'z' + 1 as u8) {
            let offset = (ch - b'a') as i32;
            let x = 16 + (offset % 8) * 72;
            let y = 48 + (offset / 8) * 64;

            let mouse_x = event_pump.mouse_state().x();
            let mouse_y = event_pump.mouse_state().y();

            if (x + 16 - mouse_x) * (x + 16 - mouse_x) + (y + 16 - mouse_y) * (y + 16 - mouse_y)
                < 16 * 16
            {
                canvas.set_draw_color(Color::YELLOW);
                canvas
                    .draw_rect(Rect::new(x, y, 32, 32))
                    .map_err(|e| e.to_string())?;

                if event_pump
                    .mouse_state()
                    .is_mouse_button_pressed(MouseButton::Left)
                    && can_click
                {
                    let encoded = encode(ch, &plugboard, &mut rotors);
                    print!("{encoded}");
                    ::std::io::stdout().flush().map_err(|e| e.to_string())?;

                    can_click = false;
                }
            }

            display_text_left_justify(
                &mut canvas,
                &texture_creator,
                x,
                y,
                &font,
                (ch as char).to_string().as_str(),
                Color::WHITE,
                16,
            )
            .map_err(|e| e.to_string())?;
        }

        //Display plugboard
        display_text_left_justify(
            &mut canvas,
            &texture_creator,
            16,
            316,
            &font,
            "Plugboard",
            Color::GREEN,
            8,
        )
        .map_err(|e| e.to_string())?;

        for ch in b'a'..(b'z' + 1 as u8) {
            let offset = (ch - b'a') as i32;
            let x = 16 + (offset % 8) * 72;
            let y = 348 + (offset / 8) * 64;

            let mouse_x = event_pump.mouse_state().x();
            let mouse_y = event_pump.mouse_state().y();

            if (x + 16 - mouse_x) * (x + 16 - mouse_x) + (y + 16 - mouse_y) * (y + 16 - mouse_y)
                < 16 * 16
            {
                canvas.set_draw_color(Color::GREEN);
                canvas
                    .draw_rect(Rect::new(x, y, 32, 32))
                    .map_err(|e| e.to_string())?;

                if event_pump
                    .mouse_state()
                    .is_mouse_button_pressed(MouseButton::Left)
                    && can_click
                {
                    can_click = false;
                    match selected_plugboard {
                        Some(c) => {
                            if c != ch {
                                match plugboard.get(&ch) {
                                    Some(c2) => {
                                        if *c2 == ch {
                                            plugboard.insert(c, ch);
                                            plugboard.insert(ch, c);
                                            selected_plugboard = None;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => match plugboard.get(&ch) {
                            Some(c2) => {
                                if *c2 == ch {
                                    selected_plugboard = Some(ch);
                                } else {
                                    plugboard.insert(*c2, *c2);
                                    plugboard.insert(ch, ch);
                                }
                            }
                            _ => {}
                        },
                    }
                }
            }

            display_text_left_justify(
                &mut canvas,
                &texture_creator,
                x,
                y,
                &font,
                (ch as char).to_string().as_str(),
                Color::WHITE,
                16,
            )
            .map_err(|e| e.to_string())?;
        }

        //Draw wires
        canvas.set_draw_color(Color::GREEN);
        for ch in plugboard.iter() {
            if ch.0 != ch.1 {
                let offset1 = (ch.0 - b'a') as i32;
                let x1 = 16 + (offset1 % 8) * 72 + 16;
                let y1 = 348 + (offset1 / 8) * 64 + 16;
                let offset2 = (ch.1 - b'a') as i32;
                let x2 = 16 + (offset2 % 8) * 72 + 16;
                let y2 = 348 + (offset2 / 8) * 64 + 16;
                canvas
                    .draw_line(Point::new(x1, y1), Point::new(x2, y2))
                    .map_err(|e| e.to_string())?;
            }
        }

        //Display rotors
        display_text_left_justify(
            &mut canvas,
            &texture_creator,
            620,
            16,
            &font,
            "Rotors",
            Color::WHITE,
            8,
        )
        .map_err(|e| e.to_string())?;

        for i in 0..rotors.len() {
            display_text_left_justify(
                &mut canvas,
                &texture_creator,
                620 + i as i32 * 32,
                48,
                &font,
                ((rotors[i].current_value + b'a') as char)
                    .to_string()
                    .as_str(),
                Color::WHITE,
                16,
            )
            .map_err(|e| e.to_string())?;
        }

        canvas.present();

        //process events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::MouseButtonUp {
                    mouse_btn: MouseButton::Left,
                    ..
                } => can_click = true,
                //Reset rotors
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    rotors[0].current_value = 0;
                    rotors[1].current_value = 0;
                    rotors[2].current_value = 0;
                    println!();
                }
                Event::KeyDown {
                    keycode: Some(k), ..
                } => {
                    if k.to_string().len() == 1 {
                        print!(
                            "{}",
                            encode(
                                k.to_string().as_bytes()[0].to_ascii_lowercase(),
                                &plugboard,
                                &mut rotors
                            )
                        );
                    }
                    ::std::io::stdout().flush().map_err(|e| e.to_string())?;
                }
                _ => {}
            }
        }
    }

    Ok(())
}
