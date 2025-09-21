//! Interactive UI for audio streaming

use super::client::AudioClient;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Interactive streaming UI with play/pause controls
pub struct StreamingUI {
    client: Arc<Mutex<AudioClient>>,
    sink: Arc<rodio::Sink>,
}

impl StreamingUI {
    /// Create new UI with audio client
    pub async fn new(client: AudioClient) -> Result<Self, Box<dyn std::error::Error>> {
        // TODO: Setup rodio::OutputStream::try_default()
        // TODO: Create rodio::Sink::try_new()
        // TODO: Print "Audio system ready"
        // TODO: Print "Press SPACE to pause/resume, 'q' to quit"
        // TODO: Wrap client in Arc<Mutex<>>
        // TODO: Wrap sink in Arc<>
        // TODO: Return StreamingUI instance
        todo!()
    }
    
    /// Start all streaming tasks and interactive controls
    pub async fn start_streaming(self) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Start chunk_fetcher_task()
        // TODO: Start audio_player_task()  
        // TODO: Start interactive_controls_task()
        // TODO: Main monitoring loop - wait for completion
        // TODO: Print "Streaming completed!"
        todo!()
    }
    
    /// Background task: fetch chunks when buffer is low
    async fn chunk_fetcher_task(client: Arc<Mutex<AudioClient>>) {
        // TODO: Loop forever
        // TODO: Check if client.needs_data() && next_chunk_id < total_chunks
        // TODO: If needs data, call client.request_chunk(next_chunk_id)
        // TODO: Print "Chunk {id} buffered ({duration}s buffered)"
        // TODO: Increment next_chunk_id
        // TODO: If buffer full or paused, sleep 100ms
        // TODO: Break on end of stream
        todo!()
    }
    
    /// Background task: play audio chunks from buffer
    async fn audio_player_task(client: Arc<Mutex<AudioClient>>, sink: Arc<rodio::Sink>) {
        // TODO: Loop forever
        // TODO: Get chunk_data = client.get_audio_chunk()
        // TODO: If got data, convert to i16 samples
        // TODO: Create rodio::buffer::SamplesBuffer with client.sample_rate, client.channels
        // TODO: Call sink.append(source)
        // TODO: If no data, sleep 50ms
        todo!()
    }
    
    /// Background task: handle SPACE pause/resume, 'q' quit
    async fn interactive_controls_task(client: Arc<Mutex<AudioClient>>, sink: Arc<rodio::Sink>) {
        // TODO: Setup termion::raw::IntoRawMode for immediate key response
        // TODO: Loop reading single bytes from stdin
        // TODO: On SPACE: toggle client.pause()/resume() and sink.pause()/play()
        // TODO: Print "⏸️ Paused" or "▶️ Resumed"
        // TODO: On 'q' or ESC: print "⏹️ Stopping..." and break
        todo!()
    }
}