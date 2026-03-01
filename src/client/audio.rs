use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use std::{
    collections::HashMap, error::Error, fs::File, io::Cursor, io::Read, path::PathBuf, sync::Arc,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Music {
    Menu,
    Lobby,
    InGame,
}

#[derive(Clone, Debug)]
pub enum Sfx {
    Click,
    GameOver,
    CardHover,
    CardShuffle,
    CardDeal,
    CardPlay,
    CardError,
    ShardPlay,
    CastMage,
    CastWitch,
    CastElf,
    CastKnight,
}

impl Sfx {
    pub fn key(&self) -> &'static str {
        match self {
            Sfx::Click => "click",
            Sfx::GameOver => "game_over",
            Sfx::CardHover => "card_hovered",
            Sfx::CardShuffle => "card_shuffle",
            Sfx::CardDeal => "card_dealed",
            Sfx::CardPlay => "card_played",
            Sfx::CardError => "card_error",
            Sfx::ShardPlay => "shard_played",
            Sfx::CastMage => "mage_cast",
            Sfx::CastWitch => "witch_cast",
            Sfx::CastElf => "elf_cast",
            Sfx::CastKnight => "knight_cast",
        }
    }
}

pub struct Audio {
    stream_handle: OutputStream,
    music_sink: Sink,
    clips: HashMap<String, Arc<Vec<u8>>>,
    music_volume: f32,
    sfx_volume: f32,
}

impl Audio {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let stream_handle = OutputStreamBuilder::open_default_stream()?;
        let music_sink = Sink::connect_new(stream_handle.mixer());
        Ok(Self {
            stream_handle,
            music_sink,
            clips: HashMap::new(),
            music_volume: 1.0,
            sfx_volume: 1.0,
        })
    }

    pub fn load_clip(
        &mut self,
        name: &str,
        path: impl Into<PathBuf>,
    ) -> Result<(), Box<dyn Error>> {
        let path = path.into();
        let mut f = File::open(&path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        self.clips.insert(name.to_string(), Arc::new(buf));
        Ok(())
    }

    fn make_decoder_from_arc(
        bytes: Arc<Vec<u8>>,
    ) -> Result<Decoder<Cursor<Vec<u8>>>, Box<dyn Error>> {
        let vec = (*bytes).clone();
        let cursor = Cursor::new(vec);
        Ok(Decoder::try_from(cursor)?)
    }

    pub fn play_sfx(&self, name: &str) {
        if let Some(bytes) = self.clips.get(name)
            && let Ok(decoder) = Self::make_decoder_from_arc(bytes.clone())
        {
            let source = decoder.buffered();
            let sink = Sink::connect_new(self.stream_handle.mixer());
            sink.set_volume(self.sfx_volume);
            sink.append(source);
            sink.detach();
        }
    }
    //for type safety and convenience
    pub fn play_sfx_enum(&self, sfx: Sfx) {
        self.play_sfx(sfx.key());
    }

    pub fn play_music(&mut self, kind: Music) {
        self.music_sink.stop();
        self.music_sink = Sink::connect_new(self.stream_handle.mixer());
        let key = match kind {
            Music::Menu => "menu",
            Music::Lobby => "lobby",
            Music::InGame => "ingame",
        };
        if let Some(bytes) = self.clips.get(key)
            && let Ok(decoder) = Self::make_decoder_from_arc(bytes.clone())
        {
            let src = decoder.repeat_infinite();
            self.music_sink.append(src);
            self.music_sink.set_volume(self.music_volume);
        }
    }

    pub fn set_music_volume(&mut self, v: f32) {
        let clamped = v.clamp(0.0, 100.0) / 100.0;
        self.music_volume = clamped;
        self.music_sink.set_volume(clamped);
    }
    pub fn set_sfx_volume(&mut self, v: f32) {
        let clamped = v.clamp(0.0, 100.0) / 100.0;
        self.sfx_volume = clamped;
    }
}
