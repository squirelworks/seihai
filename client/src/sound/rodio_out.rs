//! Device backend on `rodio` (feature `audio-rodio`).
//!
//! NOT BUILT OR RUN YET. The box that wrote this has no sound card and no
//! ALSA development files, so `cpal` cannot be built there. Treat the first
//! build on a machine with audio as a test of this file.

use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};

use super::{pan_gains, PlayRequest, SoundOut};

struct Playing {
    name: String,
    looped: bool,
    sink: Sink,
}

pub struct RodioSound {
    // The stream must stay alive for the handle to work.
    _stream: OutputStream,
    handle: OutputStreamHandle,
    playing: Vec<Playing>,
}

impl RodioSound {
    /// Open the default output device.
    pub fn open() -> Result<Self, String> {
        let (stream, handle) = OutputStream::try_default().map_err(|e| e.to_string())?;
        Ok(Self { _stream: stream, handle, playing: Vec::new() })
    }
}

impl SoundOut for RodioSound {
    fn kind(&self) -> &'static str {
        "rodio"
    }

    fn play(&mut self, req: &PlayRequest) {
        self.playing.retain(|p| p.looped || !p.sink.empty());
        let Ok(sink) = Sink::try_new(&self.handle) else {
            return;
        };
        let (l, r) = pan_gains(req.pan);
        let src = &req.pcm.samples;
        let stereo: Vec<i16> = if req.pcm.channels == 1 {
            src.iter().flat_map(|s| [(*s as f32 * l) as i16, (*s as f32 * r) as i16]).collect()
        } else {
            src.chunks_exact(2).flat_map(|f| [(f[0] as f32 * l) as i16, (f[1] as f32 * r) as i16]).collect()
        };
        let buf = SamplesBuffer::new(2, req.pcm.rate, stereo);
        if req.looped {
            sink.append(buf.repeat_infinite());
        } else {
            sink.append(buf);
        }
        self.playing.push(Playing { name: req.name.clone(), looped: req.looped, sink });
    }

    fn stop(&mut self, name: &str) {
        self.playing.retain(|p| {
            if p.name == name {
                p.sink.stop();
                false
            } else {
                true
            }
        });
    }

    fn stop_all_loops(&mut self) {
        self.playing.retain(|p| {
            if p.looped {
                p.sink.stop();
                false
            } else {
                true
            }
        });
    }
}
