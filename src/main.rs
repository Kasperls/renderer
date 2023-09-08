// https://sotrh.github.io/learn-wgpu/

use renderer::run;
use std::env;

use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, source::Source, Sink};

fn main() {
    env::set_var("RUST_BACKTRACE", "0");

    // Get a output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    // Load a sound from a file, using a path relative to Cargo.toml¨
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join("Akhiros_song.wav");
    let file = BufReader::new(File::open(path).unwrap());
    // Decode that sound file into a source
    // Play the sound directly on the device
    let audio = stream_handle.play_once(file).unwrap();
    audio.set_volume(0.00);

    pollster::block_on(run());


}

// rescources:
// https://0fps.net/2012/06/30/meshing-in-a-minecraft-game/