import 'dart:async';

import 'package:audiopc/src/dart/playback_state.dart';
import 'package:audiopc/src/rust/api/player.dart';
import 'package:audiopc/src/rust/api/renderer/output.dart';
import 'package:audiopc/src/rust/api/source.dart';
import 'package:audiopc/src/rust/api/filters.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated_io.dart';
export 'package:audiopc/src/rust/api/filters.dart';
export 'package:audiopc/src/rust/api/visualizer.dart';
export 'package:audiopc/src/rust/api/source.dart';

/// player instance
final class PcPlayer {
  /// player api
  final AudioPlayer player;
  PcPlayer._(this.player) {
    _stateController = StreamController.broadcast();
    _posController = StreamController.broadcast();

    _positionTimer = Timer.periodic(90.millis, (_) async {
      _posController.add(await player.position());
    });

    _stateTimer = Timer.periodic(90.millis, (_) async {
      _stateController.add((await player.getState()).mapState());
    });
  }

  /// Stream emit player playback state.
  Stream<PlaybackState> get stateStream => _stateController.stream;

  /// Stream emit audio position in milliseconds.
  Stream<int> get positionStream => _posController.stream;

  late final StreamController<PlaybackState> _stateController;
  late final StreamController<int> _posController;

  late final Timer _positionTimer;
  late final Timer _stateTimer;

  /// create new instance
  static PcPlayer instance() {
    final player = AudioPlayer();

    return PcPlayer._(player);
  }

  /// Return current audio duration.
  int getDuration() {
    return player.durationMillis();
  }

  /// Return duration
  int get duration => getDuration();

  /// Jump to the given position in milliseconds.
  void seek(int target) {
    player.seek(position: target);
  }

  /// Set and play the audio.
  void playSource(AudioSource source) {
    player.setSource(source: source);
    player.play();
  }

  /// Play the audio.
  void play() {
    player.play();
  }

  /// Pause the ouput stream.
  void pause() {
    player.pause();
  }

  void resume() {
    player.resume();
  }

  /// Stop audio playback.
  Future<void> stop() async {
    await player.stop();
  }

  /// Set playback speed.
  Future<void> setRate(double rate) async {
    await player.setRate(rate: rate);
  }

  /// Set volumn of ouput audio.
  Future<void> setVolumn(double volumn) async {
    await player.setVolumn(volumn: volumn);
  }

  /// Add effect to control output audio.
  Future<void> addEffect(AudioProcessor effect) async {
    await player.addEffect(effect: effect);
  }

  /// Whether audio completed or not.
  bool isCompleted() {
    return player.isCompleted();
  }

  /// Whether audio is playing.
  bool isPlaying() {
    return player.isPlaying();
  }

  /// Current samples data.
  Future<Float32List> samplesData() async {
    return await player.samplesData();
  }

  /// Return current output config.
  AudioOutputConfig getConfig() {
    return player.getOutputConfig();
  }

  /// Close the player
  void dispose() {
    _stateTimer.cancel();
    _positionTimer.cancel();
    _stateController.close();
    _posController.close();
    player.dispose();
  }
}

/// short name for create duration.
extension DurationX on int {
  /// Return duration in milliseconds.
  Duration get millis => Duration(milliseconds: this);
}
