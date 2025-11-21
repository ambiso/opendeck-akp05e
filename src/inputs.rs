use mirajazz::{error::MirajazzError, types::DeviceInput};

use crate::mappings::{ENCODER_COUNT, KEY_COUNT};

pub fn process_input(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    log::debug!("Processing input: {input}=0x{input:02x}=0b{input:08b}, {state}");

    match input {
        (1..=10) => read_button_press(input, state),
        0xa0 | 0xa1 | 0x50 | 0x51 | 0x90 | 0x91 | 0x70 | 0x71 => read_encoder_value(input),
        0x37 | 0x35 | 0x33 | 0x36 | 0x40 | 0x41 | 0x42 | 0x43 | 0x38 | 0x39 => {
            read_encoder_press(input, state)
        }
        _ => Err(MirajazzError::BadData),
    }
}

fn read_button_states(states: &[u8]) -> Vec<bool> {
    let mut bools = vec![];

    for i in 0..KEY_COUNT {
        bools.push(states[i + 1] != 0);
    }

    bools
}

fn read_button_press(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    let mut button_states = vec![0x01];
    button_states.extend(vec![0u8; KEY_COUNT + 1]);

    if input == 0 {
        return Ok(DeviceInput::ButtonStateChange(read_button_states(
            &button_states,
        )));
    }

    let pressed_index: usize = match input {
        (1..=10) => input as usize,
        // Three buttons without displays
        0x00 => 11,
        0x41 => 12,
        0x42 => 13,
        0x43 => 14,
        0x38 => 15,
        _ => return Err(MirajazzError::BadData),
    };
    if pressed_index >= 11 {
        button_states[pressed_index] = 1;
    } else {
        button_states[pressed_index] = state;
    }

    Ok(DeviceInput::ButtonStateChange(read_button_states(
        &button_states,
    )))
}

fn read_encoder_value(input: u8) -> Result<DeviceInput, MirajazzError> {
    let mut encoder_values = vec![0i8; ENCODER_COUNT];

    let (encoder, value): (usize, i8) = match input {
        // Encoder 1 (left most)
        0xa0 => (0, -1),
        0xa1 => (0, 1),
        // Encoder 2
        0x50 => (1, -1),
        0x51 => (1, 1),
        // Encoder 3
        0x90 => (2, -1),
        0x91 => (2, 1),
        // Encoder 4 (right most)
        0x70 => (3, -1),
        0x71 => (3, 1),
        _ => return Err(MirajazzError::BadData),
    };

    encoder_values[encoder] = value;
    Ok(DeviceInput::EncoderTwist(encoder_values))
}

fn read_encoder_press(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    let mut encoder_states = vec![false; ENCODER_COUNT];

    let encoder: usize = match input {
        0x37 | 0x40 => 0, // Left most
        0x35 | 0x41 => 1,
        0x33 | 0x42 => 2,
        0x36 | 0x43 => 3, // Right most
        0x38 => 4,
        0x39 => 5,
        _ => return Err(MirajazzError::BadData),
    };

    encoder_states[encoder] = state != 0;
    Ok(DeviceInput::EncoderStateChange(encoder_states))
}
