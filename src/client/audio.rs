use std::{collections::HashMap, fs::File, io::Read, io::Cursor, path::PathBuf, sync::Arc, error::Error};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};

pub enum Music { Menu, Lobby, InGame }

pub enum Sfx { click, card_place, win, lose }

impl Sfx {
    /// Return the clip key (filename key) for this SFX.
    pub fn key(&self) -> &'static str {
        match self {
            Sfx::click => "click",
            Sfx::card_place => "card_place",
            Sfx::win => "win",
            Sfx::lose => "lose",
        }
    }
}

pub struct Audio {
    // keep the output stream alive while playing
    stream_handle: OutputStream,
    music_sink: Sink,
    clips: HashMap<String, Arc<Vec<u8>>>,
    music_volume: f32,
    sfx_volume: f32,
}

impl Audio {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // use the newer rodio API that returns an `OutputStream`.
        let stream_handle = OutputStreamBuilder::open_default_stream()?;
        let music_sink = Sink::connect_new(&stream_handle.mixer());
        Ok(Self {
            stream_handle,
            music_sink,
            clips: HashMap::new(),
            music_volume: 1.0,
            sfx_volume: 1.0,
        })
    }

    pub fn load_clip(&mut self, name: &str, path: impl Into<PathBuf>) -> Result<(), Box<dyn Error>> {
        let path = path.into();
        let mut f = File::open(&path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        self.clips.insert(name.to_string(), Arc::new(buf));
        Ok(())
    }

    fn make_decoder_from_arc(bytes: Arc<Vec<u8>>) -> Result<Decoder<Cursor<Vec<u8>>>, Box<dyn Error>> {
        // Decoder requires a Read+Seek value; clone the bytes into an owned Vec for the decoder.
        // This copies but keeps the public API simple and correct.
        let vec = (&*bytes).clone();
        let cursor = Cursor::new(vec);
        Ok(Decoder::try_from(cursor)?)
    }

    pub fn play_sfx(&self, name: &str) {
        if let Some(bytes) = self.clips.get(name) {
            // create a fresh decoder per play and buffer it so playback does not borrow
            // the original bytes while decoding. Buffered source is safer for short SFX.
            if let Ok(decoder) = Self::make_decoder_from_arc(bytes.clone()) {
                let source = decoder.buffered();
                let sink = Sink::connect_new(&self.stream_handle.mixer());
                sink.set_volume(self.sfx_volume);
                sink.append(source);
                sink.detach(); // detach so it continues playing after this function returns
            }
        }
    }

    pub fn play_sfx_enum(&self, sfx: Sfx) {
        self.play_sfx(sfx.key());
    }

    pub fn play_music(&mut self, kind: Music) {
        self.music_sink.stop();
        self.music_sink = Sink::connect_new(&self.stream_handle.mixer());
        let key = match kind {
            Music::Menu => "menu",
            Music::Lobby => "lobby",
            Music::InGame => "ingame",
        };
        if let Some(bytes) = self.clips.get(key) {
            if let Ok(decoder) = Self::make_decoder_from_arc(bytes.clone()) {
                let src = decoder.repeat_infinite();
                self.music_sink.append(src);
                self.music_sink.set_volume(self.music_volume);
            }
        }
    }

    //pub fn set_music_volume(&mut self, v: f32) { self.music_volume = v; self.music_sink.set_volume(v); }
    //pub fn set_sfx_volume(&mut self, v: f32) { self.sfx_volume = v; }

    /// Temporarily mute/unmute music sink (keeps configured music_volume).
    pub fn set_music_muted(&mut self, muted: bool) {
        if muted {
            self.music_sink.set_volume(0.0);
        } else {
            self.music_sink.set_volume(self.music_volume);
        }
    }
}