use crate::APP_NAME;
use dirs::data_local_dir;
use rodio::{Decoder, OutputStreamBuilder, Sink};
use std::path::PathBuf;

fn complete_sound_file() -> PathBuf {
    data_local_dir()
        .unwrap_or_else(|| ".".into())
        .join(APP_NAME)
        .join("sounds")
        .join("complete.ogg")
}

pub fn play_complete_sound() {
    if let Ok(file) = std::fs::File::open(complete_sound_file()) {
        let stream_handle =
            OutputStreamBuilder::open_default_stream().expect("open default audio stream");

        let sink = Sink::connect_new(&stream_handle.mixer());
        let source = Decoder::try_from(file).unwrap();
        sink.append(source);
        sink.sleep_until_end();
    }
}
