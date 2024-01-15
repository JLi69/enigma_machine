use std::collections::HashMap;

const REFLECTOR: &str = "yruhqsldpxngokmiebfzcwvjat";

pub const DEFAULT_ROTOR_STATE: [Rotor; 3] = [
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

pub struct Rotor<'a> {
    pub current_value: u8,
    pub code_string: &'a str,
    pub rotate_value: u8,
}

pub fn encode(ch: u8, plugboard: &HashMap<u8, u8>, rotors: &mut [Rotor; 3]) -> char {
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
    for rotor in rotors.iter() {
        let index = ((encoded - b'a' + 26 - rotor.current_value) % 26) as usize;
        encoded = rotor.code_string.as_bytes()[index];
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

pub fn create_plugboard() -> HashMap<u8, u8> {
    let mut plugboard = HashMap::<u8, u8>::new();

    for ch in b'a'..(b'z' + 1_u8) {
        plugboard.insert(ch, ch);
    }

    plugboard
}
