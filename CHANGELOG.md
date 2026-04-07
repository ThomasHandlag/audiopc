# Audiopc lib

## 0.0.1

* Simple audio playback with WASAPI

## 0.1.0

* BREAKING: Migrated native backend from C source to Rust cdylib.
* Added CPAL-based device queries and playback controls through Dart FFI.
* Added decoding for local files and direct HTTP sources.
* Added volume and low-pass processing controls in the CPAL callback.
* Updated plugin metadata to pure ffiPlugin platforms.
