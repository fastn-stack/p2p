//! Local Audio Test
//!
//! Simple audio player to test our decoding and playback without P2P.
//! Use this to verify audio quality before testing over network.
//!
//! Usage:
//!   audio_test <audio_file>    # Play audio file locally
//!   audio_test                 # Play default SerenataGranados.ogg

use std::time::{Duration, Instant};

#[derive(clap::Parser)]
struct Args {
    /// Audio file to play (default: examples/assets/SerenataGranados.ogg)
    audio_file: Option<String>,
}

// Copy the audio loading function to avoid module dependencies
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = <Args as clap::Parser>::parse();
    
    let audio_file = args.audio_file.unwrap_or_else(|| 
        "examples/assets/SerenataGranados.ogg".to_string()
    );
    
    println!("🎵 Local Audio Test");
    println!("📁 File: {}", audio_file);
    
    // Check if file exists
    if !std::path::Path::new(&audio_file).exists() {
        eprintln!("❌ Audio file not found: {}", audio_file);
        eprintln!("💡 Make sure you're running from the fastn-p2p root directory");
        return Ok(());
    }
    
    let test_start = Instant::now();
    
    // Decode the audio file
    println!("🔍 Decoding audio file...");
    let decode_start = Instant::now();
    let (audio_data, sample_rate, channels) = match decode_audio_file(&audio_file).await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("❌ Failed to decode audio: {}", e);
            return Ok(());
        }
    };
    let decode_time = decode_start.elapsed();
    println!("✅ Audio decoded (+{:.3}s)", decode_time.as_secs_f64());
    
    // Setup audio playback
    println!("🔧 Setting up audio playback...");
    let audio_setup_start = Instant::now();
    let (_stream, stream_handle) = rodio::OutputStream::try_default()
        .map_err(|e| format!("Failed to create audio output: {}", e))?;
    
    let sink = rodio::Sink::try_new(&stream_handle)
        .map_err(|e| format!("Failed to create audio sink: {}", e))?;
    
    let audio_setup_time = audio_setup_start.elapsed();
    println!("✅ Audio system ready (+{:.3}s)", audio_setup_time.as_secs_f64());
    
    // Play the entire audio file as one source
    let playback_start = Instant::now();
    println!("🎼 Playing audio...");
    
    // Convert all PCM data to audio source
    let mut samples = Vec::with_capacity(audio_data.len() / 2);
    for chunk_bytes in audio_data.chunks_exact(2) {
        let sample = i16::from_le_bytes([chunk_bytes[0], chunk_bytes[1]]);
        samples.push(sample);
    }
    
    let source = rodio::buffer::SamplesBuffer::new(
        channels,
        sample_rate,
        samples
    );
    
    let duration_seconds = audio_data.len() as f64 / (sample_rate as f64 * channels as f64 * 2.0);
    println!("🎵 Playing {:.1}s of {}Hz {} channel audio", 
             duration_seconds, sample_rate, 
             if channels == 1 { "mono" } else { "stereo" });
    
    sink.append(source);
    
    // Wait for playback to complete
    sink.sleep_until_end();
    
    let total_time = test_start.elapsed();
    println!("✅ Playback completed (+{:.3}s total)", total_time.as_secs_f64());
    println!("");
    println!("📊 Local Playback Results:");
    println!("   🔍 Decode time: {:.3}s", decode_time.as_secs_f64());
    println!("   🔧 Setup time: {:.3}s", audio_setup_time.as_secs_f64());
    println!("   🎼 Audio duration: {:.1}s", duration_seconds);
    println!("   ⏱️  Total time: {:.3}s", total_time.as_secs_f64());
    
    if total_time.as_secs_f64() < duration_seconds + 1.0 {
        println!("   🔊 Quality: Audio played smoothly without issues");
    } else {
        println!("   ⚠️  Quality: Playback took longer than expected");
    }
    
    Ok(())
}

// Simple error type for audio operations
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("Audio file not found: {0}")]
    FileNotFound(String),
    #[error("Audio decode error: {0}")]
    DecodeError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Decode audio file (OGG/MP3) using symphonia
async fn decode_audio_file(filename: &str) -> Result<(Vec<u8>, u32, u16), AudioError> {
    println!("📁 Loading audio file: {}", filename);
    
    let file_data = tokio::fs::read(filename).await
        .map_err(|_| AudioError::FileNotFound(filename.to_string()))?;
    
    // Use symphonia for OGG files, simpler approach
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
    
    // Decode all packets with proper stereo interleaving
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
    
    // Show decode info
    let file_size_kb = file_data.len() as f64 / 1024.0;
    let pcm_size_kb = pcm_data.len() as f64 / 1024.0;
    let duration = pcm_data.len() as f64 / (sample_rate as f64 * channels as f64 * 2.0);
    
    println!("✅ Audio Info:");
    println!("   📦 File size: {:.1} KB", file_size_kb);
    println!("   🔊 PCM size: {:.1} KB", pcm_size_kb);
    println!("   ⏱️  Duration: {:.1}s", duration);
    println!("   🎵 Format: {}Hz, {} channels", sample_rate, channels);
    
    Ok((pcm_data, sample_rate, channels))
}