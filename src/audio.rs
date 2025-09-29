use rodio::{Decoder, OutputStreamBuilder, Sink};
use std::io::Cursor;
use std::thread;

const COMPLETE_SOUND: &[u8] = include_bytes!("../data/sounds/complete.ogg");

pub fn play_complete_sound() {
    let stream_handle =
        OutputStreamBuilder::open_default_stream().expect("open default audio stream");

    thread::spawn(move || {
        let sink = Sink::connect_new(&stream_handle.mixer());
        let cursor = Cursor::new(COMPLETE_SOUND);
        let source = Decoder::try_from(cursor).unwrap();
        sink.append(source);
        sink.sleep_until_end();
    });
}

#[test]
fn test_complete_sound() {
    play_complete_sound();
}
