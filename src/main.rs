use std::collections::HashMap;

mod engima_machine;
mod gui;
mod handle_input;

use engima_machine::{create_plugboard, Rotor, DEFAULT_ROTOR_STATE};
use gui::{
    clear_screen, display_keyboard, display_plugboard, display_rotors, display_title, draw_wires,
};
use handle_input::{handle_keyboard_click, handle_plugboard_click, process_events, update_rotors};

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

struct AppState<'a> {
    can_quit: bool,
    can_click: bool,
    selected_plugboard: Option<u8>,
    rotors: [Rotor<'a>; 3],
    rotor_start_pos: [u8; 3],
    plugboard: HashMap<u8, u8>,
}

const FONT_PATH: &str = "fonts/8BitOperator/8bitOperatorPlus8-Regular.ttf";

fn main() -> Result<(), String> {
    let sdl_ctx = sdl2::init().unwrap();
    let mut canvas = gui::init_canvas(&sdl_ctx)?;
    let mut event_pump = sdl_ctx.event_pump().unwrap();
    let font_ctx = sdl2::ttf::init().unwrap();
    let font = font_ctx.load_font(FONT_PATH, 32).unwrap();
    let texture_creator = canvas.texture_creator();

    let mut state = AppState {
        can_quit: false,
        can_click: false,
        selected_plugboard: None,
        rotors: DEFAULT_ROTOR_STATE,
        rotor_start_pos: [0, 0, 0],
        plugboard: create_plugboard(),
    };

    while !state.can_quit {
        clear_screen(&mut canvas);
        //Title
        display_title(&mut canvas, &texture_creator, &font)?;
        //Display keyboard
        display_keyboard(&mut canvas, &texture_creator, &font, &event_pump)?;
        //Display plugboard
        display_plugboard(&mut canvas, &texture_creator, &font, &event_pump)?;
        //Draw wires
        draw_wires(&mut canvas, &state.plugboard)?;
        //Display rotors
        display_rotors(
            &mut canvas,
            &texture_creator,
            &font,
            &event_pump,
            &state.rotors,
        )?;
        canvas.present();

        //process events
        update_rotors(
            &event_pump,
            &mut state.rotors,
            &mut state.rotor_start_pos,
            &mut state.can_click,
        );
        handle_keyboard_click(
            &event_pump,
            &mut state.can_click,
            &state.plugboard,
            &mut state.rotors,
        );
        state.selected_plugboard = handle_plugboard_click(
            &event_pump,
            &mut state.plugboard,
            state.selected_plugboard,
            &mut state.can_click,
        );
        process_events(&mut event_pump, &mut state);
    }

    println!();
    Ok(())
}
