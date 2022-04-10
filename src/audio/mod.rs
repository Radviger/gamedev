use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Error, Read, Seek};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use alto::{Alto, AltoResult, AsBufferData, Context, Mono, OutputDevice, SampleFrame, Source, SourceState, StaticSource, Stereo, StreamingSource};
use lewton::inside_ogg::OggStreamReader;
use uuid::Uuid;

pub struct SoundSystem {
    device: OutputDevice,
    context: Context,
    static_sources: HashMap<Uuid, StaticSource>,
    streaming_sources: HashMap<Uuid, StreamingSource>
}

impl SoundSystem {
    pub fn new() -> AltoResult<SoundSystem> {
        let alto = Alto::load_default()?;
        let device = alto.open(None)?;
        let context = device.new_context(None)?;
        Ok(SoundSystem {
            device, context,
            static_sources: HashMap::new(),
            streaming_sources: HashMap::new()
        })
    }

    fn cleanup(&mut self) {
        self.streaming_sources.retain(|s, e| {
            if e.state() == SourceState::Stopped {
                false
            } else {
                true
            }
        });
    }

    pub fn new_source<F, B>(&mut self, data: B, looping: bool) -> AltoResult<StaticSource> where F: SampleFrame, B: AsBufferData<F> {
        let buffer = self.context.new_buffer(data, 44_000)?;
        let buffer = Arc::new(buffer);
        let mut source = self.context.new_static_source()?;
        source.set_buffer(buffer)?;
        source.set_looping(looping);
        Ok(source)
    }

    pub fn play<F, B>(&mut self, data: B, looping: bool) -> Result<Uuid, PlaybackError> where F: SampleFrame, B: AsBufferData<F> {
        let mut source = self.new_source(data, looping)?;
        source.play();
        let id = Uuid::new_v4();
        self.static_sources.insert(id, source);
        Ok(id)
    }

    pub fn play_streaming_bytes(&mut self, bytes: &[u8]) -> Result<Uuid, PlaybackError> {
        let reader = OggStreamReader::new(Cursor::new(bytes))?;
        self.play_streaming_reader(reader)
    }

    pub fn play_streaming_file<P>(&mut self, path: P) -> Result<Uuid, PlaybackError> where P: AsRef<Path> {
        let path = path.as_ref();
        let file = File::open(path)?;
        let reader = OggStreamReader::new(file)?;
        self.play_streaming_reader(reader)
    }

    pub fn play_streaming_reader<T>(&mut self, mut reader: OggStreamReader<T>) -> Result<Uuid, PlaybackError> where T: Read + Seek {
        self.cleanup();
        let mut n = 0;
        let mut len_play = 0.0;
        let sample_rate = reader.ident_hdr.audio_sample_rate as i32;
        let sample_channels = reader.ident_hdr.audio_channels as f32
            * reader.ident_hdr.audio_sample_rate as f32;

        let mut source = self.context.new_streaming_source()?;

        while let Some(samples) = reader.read_dec_packet_itl()? {
            n += 1;
            let buffer = match reader.ident_hdr.audio_channels {
                1 => self.context.new_buffer::<Mono<i16>, _>(&samples, sample_rate)?,
                2 => self.context.new_buffer::<Stereo<i16>, _>(&samples, sample_rate)?,
                other => panic!("Unsupported number of channels: {}", other)
            };
            source.queue_buffer(buffer)?;
            len_play += samples.len() as f32 / sample_channels;
        }
        source.play();
        let id = Uuid::new_v4();
        self.streaming_sources.insert(id, source);
        Ok(id)
    }

    pub fn is_playing(&self, id: &Uuid) -> Option<bool> {
        let source = self.static_sources.get(id)?;
        Some(source.state() == SourceState::Playing)
    }

    pub fn is_paused(&self, id: &Uuid) -> Option<bool> {
        let source = self.static_sources.get(id)?;
        Some(source.state() == SourceState::Paused)
    }

    pub fn stop(&mut self, id: &Uuid) -> Option<bool> {
        let mut source = self.static_sources.get_mut(id)?;
        Some(if source.state() == SourceState::Playing {
            source.stop();
            true
        } else {
            false
        })
    }
}

#[derive(Debug)]
pub enum PlaybackError {
    Io(std::io::Error),
    Al(alto::AltoError),
    Vorbis(lewton::VorbisError)
}

impl From<std::io::Error> for PlaybackError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<alto::AltoError> for PlaybackError {
    fn from(e: alto::AltoError) -> Self {
        Self::Al(e)
    }
}

impl From<lewton::VorbisError> for PlaybackError {
    fn from(e: lewton::VorbisError) -> Self {
        Self::Vorbis(e)
    }
}