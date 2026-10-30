//! Sound output.
//!
//! The sound rules, the `.wav` decoder and the null and recording backends
//! are in `seihai_core::sound` and re-exported here. This module adds the
//! real device backend, behind the `audio-rodio` cargo feature, and picks
//! the backend the client starts with.

pub use seihai_core::sound::*;

#[cfg(feature = "audio-rodio")]
mod rodio_out;
#[cfg(feature = "audio-rodio")]
pub use rodio_out::RodioSound;

/// The best backend this build has: the device one when the feature is on
/// and a device opens, otherwise [`NullSound`]. The string says which and
/// why, for one startup log line.
pub fn default_backend() -> (Box<dyn SoundOut>, String) {
    #[cfg(feature = "audio-rodio")]
    {
        match RodioSound::open() {
            Ok(b) => return (Box::new(b), "audio device opened".to_string()),
            Err(e) => return (Box::new(NullSound), format!("no audio device ({e}); sound is silent")),
        }
    }
    #[cfg(not(feature = "audio-rodio"))]
    {
        (Box::new(NullSound), "built without the audio-rodio feature; sound is silent".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_backend_without_the_feature_is_null() {
        let (b, why) = default_backend();
        if cfg!(feature = "audio-rodio") {
            assert!(!why.is_empty());
        } else {
            assert_eq!(b.kind(), "null");
            assert!(why.contains("audio-rodio"));
        }
    }
}
