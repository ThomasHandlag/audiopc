# audiopc

Rust-powered Flutter FFI audio plugin with a CPAL backend for playback and processing.

The plugin is designed for Flutter apps that need native audio output with support for local files, direct internet URLs, common codec decoding through Symphonia, and simple in-callback processing such as volume and low-pass control.

## Features

- Play local audio files through CPAL.
- Stream audio from direct internet URLs.
- Decode common formats through Symphonia.
- Apply volume and low-pass processing in the audio callback.
- Query the active output device configuration.
