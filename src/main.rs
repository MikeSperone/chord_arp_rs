extern crate midir;

use std::io::{stdin, stdout, Write};
use std::error::Error;

use midir::{MidiInput, MidiOutput, Ignore};

// #[derive(Eq, Copy, Clone)]
// struct HeldKeys {
//     pc_collection: Vec<u8>,
// };

// impl HeldKeys {
//     fn addNote(&self, pc) -> vec {
//         self.pc_collection.push(pc);
//     }

//     fn removeNote(&self, pc) -> vec {
//         if self.pc_collection.contains(&pc) {
//             let found_index = pc_collection.iter().position(|&r| r == pc).unwrap();
//             pc_collection.swap_remove(found_index);
//         }
    // }
// }

fn main() {
    println!("Running MIDI forwarding script");
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err.description())
    }
}

fn run() -> Result<(), Box<Error>> {
    let mut input = String::new();

    let mut midi_in = MidiInput::new("midir forwarding input")?;
    midi_in.ignore(Ignore::None);
    let midi_out = MidiOutput::new("midir forwarding output")?;

    println!("Available input ports:");
    for i in 0..midi_in.port_count() {
        println!("{}: {}", i, midi_in.port_name(i)?);
    }
    print!("Please select input port: ");
    stdout().flush()?;
    stdin().read_line(&mut input)?;
    let in_port: usize = input.trim().parse()?;

    println!("\nAvailable output ports:");
    for i in 0..midi_out.port_count() {
        println!("{}: {}", i, midi_out.port_name(i)?);
    }
    println!("Please select output port:");
    stdout().flush()?;
    input.clear();
    stdin().read_line(&mut input)?;
    let out_port: usize = input.trim().parse()?;

    let in_port_name = midi_in.port_name(in_port)?;
    let out_port_name = midi_out.port_name(out_port)?;

    println!("\n------------------------------------------------");
    println!("\n  Reading MIDI in on port {:?}", in_port_name);
    println!("\n  Sending MIDI out on port {:?}", out_port_name);
    println!("\n------------------------------------------------");

    let mut conn_out = midi_out.connect(out_port, "midir-forward")?;

    let mut pc_bucket = Vec::new();

    let _conn_in = midi_in.connect(in_port, "midir-forward", move |_stamp, message, _| {
        conn_out.send(message).unwrap_or_else(|_| println!("Error when forwarding message ..."));
        let pitch = message[1];
        let velocity = message[2];
        let pitch_class = pitch % 12;

        if velocity == 0 {
            println!("Remove {}", pitch_class);
            if pc_bucket.contains(&pitch_class) {
                let found_index = pc_bucket.iter().position(|&r| r == pitch_class).unwrap();
                pc_bucket.swap_remove(found_index);
            }
        } else {
            pc_bucket.push(pitch_class);
        }
        println!("keys pressed: {:?}", pc_bucket);
        // println!("pitch: {}, (pc: {}), velocity: {}", pitch, pitch_class, velocity);
    }, ())?;

    println!("Connections open, forwarding from '{}' to '{}' (press enter to exit) ...", in_port_name, out_port_name);

    input.clear();
    stdin().read_line(&mut input)?;

    println!("Closing connections");
    Ok(())
}
