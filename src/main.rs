extern crate midir;
extern crate rosc;

// use std::{env};
use std::io::{stdin, stdout, Write};
use std::error::Error;

// For MIDI
use midir::{MidiInput, MidiOutput, Ignore};

// For OSC
// use std::{env};
use std::net::{UdpSocket, SocketAddrV4};
use std::str::FromStr;
use rosc::{OscPacket, OscMessage, OscType};
use rosc::encoder;

// For CLI
extern crate term;
// use std::io::prelude::*;

#[derive(Clone)]
struct HeldKeys {
    collection: Vec<u8>,
}

impl HeldKeys {

    fn new() -> HeldKeys {
        HeldKeys {
            collection: Vec::new()
        }
    }

    fn add_note(&mut self, pc: u8) {
        self.collection.push(pc);
    }

    fn remove_note(&mut self, pc: u8) {
        if self.collection.contains(&pc) {
            let found_index = self.collection.iter().position(|&r| r == pc).unwrap();
            self.collection.swap_remove(found_index);
        }
    }

}

fn main() {
    println!("Running MIDI forwarding script");
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err.description())
    }
}

fn clear_screen() {
    let mut t = term::stdout().unwrap();
    t.fg(term::color::WHITE).unwrap();
    t.reset().unwrap();
    print!("{}[2J", 27 as char);
    print!("{}[0;0H", 27 as char);
}

fn run() -> Result<(), Box<Error>> {

    let mut t = term::stdout().unwrap();
    clear_screen();
    stdout().flush().unwrap();

    // ------------------------------
    //  1. Get MIDI IO
    // ------------------------------
    let mut input = String::new();

    let mut midi_in = MidiInput::new("midir forwarding input")?;
    midi_in.ignore(Ignore::All);
    let midi_out = MidiOutput::new("midir forwarding output")?;


    if midi_in.port_count() == 0 {
        println!("You have no MIDI inputs available.");
        return Ok(())
    }

    // Input ports
    println!("Available input ports:");
    for i in 0..midi_in.port_count() {
        println!("{}: {}", i, midi_in.port_name(i)?);
    }
    print!("Please select input port: ");
    stdout().flush()?;
    stdin().read_line(&mut input)?;
    let in_port: usize = input.trim().parse()?;
    let in_port_name = midi_in.port_name(in_port)?;

    // Output
    print!("Would you like to forward MIDI to another destination? [Y|y]");
    stdout().flush()?;
    input.clear();
    stdin().read_line(&mut input)?;
    let ans: char = input.trim().parse()?;
    let mut is_forwarding_midi: bool = false;
    let mut out_port: usize = 0;
    let mut out_port_name = String::from("default");
    let positive_response: &str = "Yy";
    if positive_response.contains(ans) {

        is_forwarding_midi = true;
        println!("\nAvailable output ports:");
        for i in 0..midi_out.port_count() {
            println!("{}: {}", i, midi_out.port_name(i)?);
        }
        println!("Please select output port:");
        stdout().flush()?;
        input.clear();
        stdin().read_line(&mut input)?;
        out_port = input.trim().parse()?;
        out_port_name = midi_out.port_name(out_port)?;

    }

    clear_screen();
    t.fg(term::color::BLUE).unwrap();
    writeln!(t, "\n------------------------------------------------").unwrap();
    writeln!(t, "\n  Reading MIDI in on port {:?}", in_port_name).unwrap();
    if is_forwarding_midi {
        writeln!(t, "\n  Sending MIDI out on port {:?}", out_port_name).unwrap();
    }
    writeln!(t, "\n------------------------------------------------").unwrap();
    t.reset().unwrap();

    // ------------------------------
    //  2. Forward MIDI
    // ------------------------------

    let from_address = "127.0.0.1:57000";
    let to_address = "127.0.0.1:57001";
    let my_host_name = SocketAddrV4::from_str(&from_address).unwrap();
    let destination_host_name = SocketAddrV4::from_str(&to_address).unwrap();
    let sock = UdpSocket::bind(my_host_name).unwrap();

    let mut hk = HeldKeys::new();
    let mut conn_out = midi_out.connect(out_port, "midir-forward")?;
    let _conn_in = midi_in.connect(in_port, "midir-forward", move |_stamp, message, _| {
        if is_forwarding_midi {
            conn_out
                .send(message)
                .unwrap_or_else(|_| println!("Error when forwarding message ..."));
        }
        let midi_event = message[0];
        let pitch = message[1];
        // let velocity = message[2];
        let pitch_class = pitch % 12;

        if midi_event > 127 && midi_event < 144 {
            // note off events
            hk.remove_note(pitch_class);
        } else if midi_event > 143 && midi_event < 160 {
            // note on events
            hk.add_note(pitch_class);
        }

        // ------------------------------
        //  3. Send Via UDP
        // ------------------------------
        let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
            addr: "/pc".to_string(),
            args: Some(vec![OscType::Blob(hk.collection.to_vec())]),
        })).unwrap();

        sock.send_to(&msg_buf, destination_host_name).unwrap();

        t.cursor_up().unwrap();
        t.delete_line().unwrap();
        t.fg(term::color::GREEN).unwrap();
        writeln!(t, "{:?}", hk.collection).unwrap();
    }, ())?;

    print!("Connections open...");
    if is_forwarding_midi {
        print!("forwarding from '{}' to '{}' ", in_port_name, out_port_name);
    }
    print!("(press enter to exit) ...\n\r");
    println!("\n\r");
    input.clear();
    stdin().read_line(&mut input)?;

    clear_screen();
    println!("Goodbye.");
    Ok(())
}
