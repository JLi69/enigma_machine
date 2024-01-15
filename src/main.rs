use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;
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

fn create_text_texture<'a>(
    text: &str,
    texture_creator: &'a TextureCreator<WindowContext>,
    font: &Font,
    col: Color,
) -> Result<(Texture<'a>, u32), String> {
    let font_surface = font.render(text).solid(col).map_err(|e| e.to_string())?;
    let font_texture = texture_creator
        .create_texture_from_surface(&font_surface)
        .map_err(|e| e.to_string())?;
    Ok((font_texture, text.len() as u32))
}

fn display_text(
    canvas: &mut Canvas<Window>,
    font_texture: &Texture,
    len: u32,
    x: i32,
    y: i32,
    char_sz: u32,
) -> Result<(), String> {
    let text_rect = Rect::new(x, y, char_sz * len, char_sz * 2);
    canvas
        .copy(font_texture, None, text_rect)
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
    if let Some(pair) = plugboard.get(&ch) {
        encoded = *pair;
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
    if let Some(pair) = plugboard.get(&encoded) {
        encoded = *pair;
    }

    //Return the encoded character
    encoded as char
}

fn create_plugboard() -> HashMap<u8, u8> {
    let mut plugboard = HashMap::<u8, u8>::new();

    for ch in b'a'..(b'z' + 1 as u8) {
        plugboard.insert(ch, ch);
    }

    plugboard
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
    let mut rotor_start_pos = [0, 0, 0];
    let mut plugboard = create_plugboard();

    'running: loop {
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        //Title
        let (texture, len) =
            create_text_texture("Enigma Machine", &texture_creator, &font, Color::YELLOW)?;
        display_text(&mut canvas, &texture, len, 16, 16, 8)?;
        //Display keyboard
        display_keyboard(&mut canvas, &texture_creator, &font, &event_pump)?;
        //Display plugboard
        display_plugboard(&mut canvas, &texture_creator, &font, &event_pump)?;
        //Draw wires
        draw_wires(&mut canvas, &plugboard)?;
        //Display rotors
        display_rotors(&mut canvas, &texture_creator, &font, &event_pump, &rotors)?;

        canvas.present();

        //process events
        update_rotors(
            &event_pump,
            &mut rotors,
            &mut rotor_start_pos,
            &mut can_click,
        );
        handle_keyboard_click(&event_pump, &mut can_click, &plugboard, &mut rotors);
        selected_plugboard = handle_plugboard_click(
            &event_pump,
            &mut plugboard,
            selected_plugboard,
            &mut can_click,
        );
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
                } => reset_rotors(&mut rotors, &rotor_start_pos),
                Event::KeyDown {
                    keycode: Some(k), ..
                } => encode_and_output(k, &plugboard, &mut rotors),
                _ => {}
            }
        }
    }

    println!();
    Ok(())
}

fn reset_rotors(rotors: &mut [Rotor; 3], rotor_start_pos: &[u8; 3]) {
    for (i, rotor) in rotors.iter_mut().enumerate() {
        rotor.current_value = rotor_start_pos[i];
    }
    println!();
}

fn encode_and_output(k: Keycode, plugboard: &HashMap<u8, u8>, rotors: &mut [Rotor; 3]) {
    if k.to_string().len() != 1 {
        return;
    }

    let key_ch = k.to_string().as_bytes()[0].to_ascii_lowercase();
    if key_ch >= b'a' && key_ch <= b'z' {
        let encoded = encode(key_ch, plugboard, rotors);
        print!("{}", encoded);
        ::std::io::stdout()
            .flush()
            .map_err(|e| e.to_string())
            .expect("Failed to flush stdout!");
    }
}

fn draw_wires(canvas: &mut Canvas<Window>, plugboard: &HashMap<u8, u8>) -> Result<(), String> {
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

    Ok(())
}

fn dist(x1: i32, y1: i32, x2: i32, y2: i32) -> f64 {
    (((x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2)) as f64).sqrt()
}

fn display_rotors(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    font: &Font,
    event_pump: &EventPump,
    rotors: &[Rotor; 3],
) -> Result<(), String> {
    let (texture, len) = create_text_texture("Rotors", texture_creator, font, Color::WHITE)?;
    display_text(canvas, &texture, len, 620, 16, 8)?;

    for i in 0..rotors.len() {
        let x = 620 + i as i32 * 32;
        let y = 48;

        let mouse_x = event_pump.mouse_state().x();
        let mouse_y = event_pump.mouse_state().y();

        if dist(x + 16, y + 16, mouse_x, mouse_y) < 16.0 {
            canvas.set_draw_color(Color::CYAN);
            canvas
                .draw_rect(Rect::new(x, y, 32, 32))
                .map_err(|e| e.to_string())?;
        }

        let text = ((rotors[i].current_value + b'a') as char).to_string();
        let (texture, len) = create_text_texture(&text, &texture_creator, &font, Color::WHITE)?;
        display_text(canvas, &texture, len, x, y, 16)?;
    }

    Ok(())
}

fn update_rotors(
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

fn display_keyboard(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    font: &Font,
    event_pump: &EventPump,
) -> Result<(), String> {
    for ch in b'a'..(b'z' + 1 as u8) {
        let offset = (ch - b'a') as i32;
        let x = 16 + (offset % 8) * 72;
        let y = 48 + (offset / 8) * 64;

        let mouse_x = event_pump.mouse_state().x();
        let mouse_y = event_pump.mouse_state().y();

        if dist(x + 16, y + 16, mouse_x, mouse_y) < 16.0 {
            canvas.set_draw_color(Color::YELLOW);
            canvas
                .draw_rect(Rect::new(x, y, 32, 32))
                .map_err(|e| e.to_string())?;
        }

        let text = (ch as char).to_string();
        let (texture, len) = create_text_texture(&text, texture_creator, font, Color::WHITE)?;
        display_text(canvas, &texture, len, x, y, 16)?;
    }

    Ok(())
}

fn handle_keyboard_click(
    event_pump: &EventPump,
    can_click: &mut bool,
    plugboard: &HashMap<u8, u8>,
    rotors: &mut [Rotor; 3],
) {
    for ch in b'a'..(b'z' + 1 as u8) {
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

fn display_plugboard(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    font: &Font,
    event_pump: &EventPump,
) -> Result<(), String> {
    let (texture, len) = create_text_texture("Plugboard", texture_creator, font, Color::GREEN)?;
    display_text(canvas, &texture, len, 16, 316, 8)?;

    for ch in b'a'..(b'z' + 1 as u8) {
        let offset = (ch - b'a') as i32;
        let x = 16 + (offset % 8) * 72;
        let y = 348 + (offset / 8) * 64;

        let mouse_x = event_pump.mouse_state().x();
        let mouse_y = event_pump.mouse_state().y();

        if dist(x + 16, y + 16, mouse_x, mouse_y) < 16.0 {
            canvas.set_draw_color(Color::GREEN);
            canvas
                .draw_rect(Rect::new(x, y, 32, 32))
                .map_err(|e| e.to_string())?;
        }

        let text = (ch as char).to_string();
        let (texture, len) = create_text_texture(&text, &texture_creator, &font, Color::WHITE)?;
        display_text(canvas, &texture, len, x, y, 16)?;
    }

    Ok(())
}

fn key_equal(map: &HashMap<u8, u8>, ch: u8) -> bool {
    if let Some(c2) = map.get(&ch) {
        if *c2 == ch {
            return true;
        }
    }

    false
}

fn handle_plugboard_click(
    event_pump: &EventPump,
    plugboard: &mut HashMap<u8, u8>,
    selected_plugboard: Option<u8>,
    can_click: &mut bool,
) -> Option<u8> {
    for ch in b'a'..(b'z' + 1 as u8) {
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
        let has_plug = key_equal(&plugboard, ch);

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
