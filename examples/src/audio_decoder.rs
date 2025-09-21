//! Shared audio decoding functionality

// Audio decoding error types
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("Audio file not found: {0}")]
    FileNotFound(String),
    #[error("Audio decode error: {0}")]
    DecodeError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Decode audio file (OGG/MP3) to PCM data
pub async fn decode_audio_file(filename: &str) -> Result<(Vec<u8>, u32, u16), AudioError> {
    let file_data = tokio::fs::read(filename).await
        .map_err(|_| AudioError::FileNotFound(filename.to_string()))?;
    
    // Use symphonia for OGG files
    use symphonia::core::audio::{AudioBufferRef, Signal};
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;
    
    let file_data_owned = file_data.to_vec();
    let cursor = std::io::Cursor::new(file_data_owned);
    let mss = MediaSourceStream::new(Box::new(cursor), Default::default());
    
    let hint = Hint::new();
    let meta_opts = MetadataOptions::default();
    let fmt_opts = FormatOptions::default();
    
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .map_err(|e| AudioError::DecodeError(format!("Format probe failed: {:?}", e)))?;
    
    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or_else(|| AudioError::DecodeError("No supported audio tracks found".to_string()))?;
    
    let dec_opts = DecoderOptions::default();
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &dec_opts)
        .map_err(|e| AudioError::DecodeError(format!("Decoder creation failed: {:?}", e)))?;
    
    let track_id = track.id;
    let mut pcm_data = Vec::new();
    let mut sample_rate = 44100;
    let mut channels = 2;
    
    // Decode with proper stereo interleaving
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => {
                return Err(AudioError::DecodeError(format!("Packet read error: {:?}", e)));
            }
        };
        
        if packet.track_id() != track_id {
            continue;
        }
        
        match decoder.decode(&packet) {
            Ok(decoded) => {
                sample_rate = decoded.spec().rate;
                channels = decoded.spec().channels.count() as u16;
                
                // Proper stereo interleaving: [L,R,L,R,L,R...]
                match decoded {
                    AudioBufferRef::F32(buf) => {
                        let channels_count = buf.spec().channels.count();
                        let frames = buf.frames();
                        
                        for frame_idx in 0..frames {
                            for ch in 0..channels_count {
                                let sample = buf.chan(ch)[frame_idx];
                                let sample_i16 = (sample * 32767.0).clamp(-32767.0, 32767.0) as i16;
                                pcm_data.extend_from_slice(&sample_i16.to_le_bytes());
                            }
                        }
                    }
                    AudioBufferRef::S16(buf) => {
                        let channels_count = buf.spec().channels.count();
                        let frames = buf.frames();
                        
                        for frame_idx in 0..frames {
                            for ch in 0..channels_count {
                                let sample = buf.chan(ch)[frame_idx];
                                pcm_data.extend_from_slice(&sample.to_le_bytes());
                            }
                        }
                    }
                    _ => {
                        return Err(AudioError::DecodeError("Unsupported audio format".to_string()));
                    }
                }
            }
            Err(symphonia::core::errors::Error::IoError(_)) => break,
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => {
                return Err(AudioError::DecodeError(format!("Decode error: {:?}", e)));
            }
        }
    }
    
    if pcm_data.is_empty() {
        return Err(AudioError::DecodeError("No audio data decoded".to_string()));
    }
    
    Ok((pcm_data, sample_rate, channels))
}