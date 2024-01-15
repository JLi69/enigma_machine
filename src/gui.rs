use crate::Rotor;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;
use std::collections::HashMap;

fn dist(x1: i32, y1: i32, x2: i32, y2: i32) -> f64 {
    (((x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2)) as f64).sqrt()
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

pub fn display_plugboard(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    font: &Font,
    event_pump: &EventPump,
) -> Result<(), String> {
    let (texture, len) = create_text_texture("Plugboard", texture_creator, font, Color::GREEN)?;
    display_text(canvas, &texture, len, 16, 316, 8)?;

    for ch in b'a'..(b'z' + 1_u8) {
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
        let (texture, len) = create_text_texture(&text, texture_creator, font, Color::WHITE)?;
        display_text(canvas, &texture, len, x, y, 16)?;
    }

    Ok(())
}

pub fn display_keyboard(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    font: &Font,
    event_pump: &EventPump,
) -> Result<(), String> {
    for ch in b'a'..(b'z' + 1_u8) {
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

pub fn display_rotors(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    font: &Font,
    event_pump: &EventPump,
    rotors: &[Rotor; 3],
) -> Result<(), String> {
    let (texture, len) = create_text_texture("Rotors", texture_creator, font, Color::WHITE)?;
    display_text(canvas, &texture, len, 620, 16, 8)?;

    for (i, rotor) in rotors.iter().enumerate() {
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

        let text = ((rotor.current_value + b'a') as char).to_string();
        let (texture, len) = create_text_texture(&text, texture_creator, font, Color::WHITE)?;
        display_text(canvas, &texture, len, x, y, 16)?;
    }

    Ok(())
}

pub fn draw_wires(canvas: &mut Canvas<Window>, plugboard: &HashMap<u8, u8>) -> Result<(), String> {
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

pub fn display_title(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    font: &Font,
) -> Result<(), String> {
    let (texture, len) =
        create_text_texture("Enigma Machine", texture_creator, font, Color::YELLOW)?;
    display_text(canvas, &texture, len, 16, 16, 8)?;
    Ok(())
}

pub fn init_canvas(ctx: &sdl2::Sdl) -> Result<Canvas<Window>, String> {
    let video_subsystem = ctx.video().unwrap();
    let window = video_subsystem
        .window("Enigma Machine", 750, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())
}

pub fn clear_screen(canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
}
