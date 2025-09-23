# Client-Driven Audio Streaming Design

## Overview
Complete end-to-end analysis of client-driven audio streaming with buffer management and interactive controls.

## Architecture

```
┌─────────────────┐    ┌─────────────────┐
│  Audio Client   │    │  Audio Server   │
│                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │   Buffer    │ │    │ │ Audio Data  │ │
│ │ Management  │ │◄──►│ │   Cache     │ │
│ └─────────────┘ │    │ └─────────────┘ │
│                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │ Interactive │ │    │ │   Request   │ │
│ │ Controls    │ │    │ │  Handlers   │ │
│ │ (SPACE/q)   │ │    │ │             │ │
│ └─────────────┘ │    │ └─────────────┘ │
└─────────────────┘    └─────────────────┘
```

## Protocol Design

### Option 1: String Constants (Recommended)
```rust
pub const AUDIO_GET_INFO: &'static str = "audio.get_info";
pub const AUDIO_REQUEST_CHUNK: &'static str = "audio.request_chunk";
```

### Option 2: Use Current Enum Approach
```rust
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum AudioProtocol {
    GetInfo,
    RequestChunk,
}
```

**Decision**: Start with enum approach (current fastn-p2p API), migrate to string constants later.

## Data Flow

1. **Client connects** → Server loads audio file (once)
2. **Client requests info** → Server returns metadata (duration, chunks, format)
3. **Client starts buffer loop** → Requests chunks when buffer < 3s
4. **Client starts playback loop** → Plays chunks from buffer
5. **Client interactive loop** → SPACE pauses (stops requesting), resumes (starts requesting)

## Performance Targets

- **Buffer target**: 3 seconds of audio
- **Chunk size**: 256KB (~3 seconds of audio per chunk)
- **Request frequency**: Only when buffer < target
- **Pause mechanism**: Stop requesting chunks, drain buffer
- **Resume mechanism**: Start requesting chunks again

## Module Structure

- `protocol.rs` - Request/response types
- `server.rs` - Audio server with chunk serving
- `client.rs` - Buffer management and P2P requests
- `ui.rs` - Interactive controls and audio playback
- `main.rs` - Entry point orchestration