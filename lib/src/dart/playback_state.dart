/// Map playback state.
enum PlaybackState {
  /// Initial state
  idle,

  /// Ready for playing
  buffering,

  /// Audio is playing, provide neccessary data
  playing,

  ///
  paused,

  /// Decode is paused and do not emit new data
  stopped,

  /// End of stream
  completed,
}

/// Map state from rust.
extension PlaybackStateX on int {
  /// Return state from rust.
  PlaybackState mapState() {
    return PlaybackState.values[this];
  }
}
