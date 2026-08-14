import 'dart:async';

import 'package:audiopc/audiopc.dart';
import 'package:audiopc/src/rust/api/player.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated_io.dart';

/// player instance
final class PcPlayer {
  /// player api
  final AudioPlayer player;

  PcPlayer._(this.player) {
    _stateController = StreamController.broadcast();
    _posController = StreamController.broadcast();

    _positionTimer = Timer.periodic(90.millis, (_) {
      _posController.add(player.position());
    });

    _stateTimer = Timer.periodic(90.millis, (_) {
      final state = player.getState().mapState();
      _stateController.add(state);
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

  /// Create new instance.
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

  /// Set source to player.
  void setSource(AudioSource source) {
    player.setSource(source: source);
  }

  List<BandEnergy> getBands(List<VBandKind> kinds) {
    return player.getBand(kinds: kinds);
  }

  /// Resume audio playback.
  void resume() {
    player.resume();
  }

  /// Stop audio playback.
  void stop() {
    player.stop();
  }

  /// Set playback speed.
  void setRate(double rate) {
    player.setRate(rate: rate);
  }

  /// Set 10 bands equalizer.
  void set10BandsEqualizer() {
    player.set10BandEq();
  }

  /// Set 8 bands equalizer.
  void set8BandsEqualizer() {
    player.set8BandEq();
  }

  /// Manually add equalizer band.
  void setBand(int index, double gainDb) {
    player.setBandGain(index: index, gain: gainDb);
  }

  /// Enable(if disabled) and control the effect.
  void setReverb(double roomSize, double damping, double wet) {
    player.setReverb(roomSize: roomSize, damping: damping, wet: wet);
  }

  void setChorus(double rate, double depth, double wet) {
    player.setChorus(rate: rate, depth: depth, wet: wet);
  }

  void setDelay(double delaySeconds, double feedback, double wet) {
    player.setDelay(delaySeconds: delaySeconds, feedback: feedback, wet: wet);
  }

  void setOverDride(double drive, double output) {
    player.setOverDride(drive: drive, output: output);
  }

  void setGain(double gain) {
    player.setGain(gain: gain);
  }

  void setThreshold(double threshold) {
    player.setThreshold(threshold: threshold);
  }

  void disableThreshold() {
    player.disableThreshold();
  }

  void disableOverDrive() {
    player.disableOverDrive();
  }

  void disableGain() {
    player.disableGain();
  }

  void disableCompressor() {
    player.disableCompressor();
  }

  void disableReverb() {
    player.disableReverb();
  }

  void disableDelay() {
    player.disableDelay();
  }


  void disableChorus() {
    player.disableChorus();
  }

  void setCompressor(
    double thresholdDb,
    double ratio,
    double attackS,
    double releaseS,
    double makeupDb,
    double sampleRate,
  ) {
    player.setCompressor(
      thresholdDb: thresholdDb,
      ratio: ratio,
      attackS: attackS,
      releaseS: releaseS,
      makeupDb: makeupDb,
      sampleRate: sampleRate,
    );
  }

  /// Set volumn of ouput audio.
  void setVolumn(double volumn) {
    player.setVolumn(volumn: volumn);
  }

  /// Whether audio completed or not.
  bool isCompleted() {
    return player.isCompleted();
  }

  /// Whether audio is playing.
  bool isPlaying() {
    return player.isPlaying();
  }

  /// Get computed bars. The return data sized is determined by processor.
  ///
  /// If audio processor did not set yet then an exception will be throw.
  /// To set how many bars and precise the bars is just call [setProcesor].
  Float32List getBars() {
    return player.getBars();
  }

  Float32List getSamples(int buffSize) {
    return player.getSamples(buffSize: buffSize);
  }

  /// Create new processor with bar count and fft size.
  ///
  /// Increase [fftSize] can help visualizer bar more presice,
  /// but should consider performance if it too large.
  void setProcesor(int barCount, int fftSize) {
    player.setProcessor(barCount: barCount, fftSize: fftSize);
  }

  double drumBeat() {
    return VisualizerProcessor.drumBeat(
      bands: getBands([.kick, .snareBody, .snareCrack]),
    );
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
