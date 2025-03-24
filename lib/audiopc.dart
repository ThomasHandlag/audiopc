import 'dart:async';

import 'package:audiopc/audiopc_platform.dart';
import 'package:audiopc/audiopc_state.dart';
import 'package:audiopc/audopc_helper.dart';
import 'package:audiopc/audio_metadata.dart';
import 'package:audiopc/player_event.dart';

class Audiopc {
  final _platform = AudiopcPlatform();
  final String id;

  double duration = 0.0;

  AudiopcState _state = AudiopcState.none;

  AudiopcState get state => _state;

  bool isCompleted = false;

  PositionListener? _positionListener;
  SamplesListener? _samplesListener;

  Stream<double> get onPositionChanged =>
      _positionListener!.streamControler.stream;
  Stream<List<double>> get onSamples =>
      _samplesListener!.streamControler.stream;

  StreamSubscription<dynamic>? _eventSubscription;

  final _eventStreamController = StreamController<PlayerEvent>.broadcast();

  Stream<double> get onDurationChanged => _eventStreamController.stream
          .where((event) => event.type == PlayerEventType.duration)
          .map((event) {
        duration = event.value as double;
        return event.value as double;
      });

  // Stream<String> get onError => _eventStreamController.stream
  //     .where((event) => event.type == PlayerEventType.error)
  //     .map((event) => event.value as String);

  Stream<AudiopcState> get onStateChanged => _eventStreamController.stream
          .where((event) => event.type == PlayerEventType.state)
          .map((event) {
        final stateValue = event.value as double;
        if (state == AudiopcState.playing && stateValue == 5.0) {
          isCompleted = true;
        } else {
          isCompleted = false;
        }

        switch (stateValue) {
          case 3.0:
            {
              _positionListener!.start();
              _samplesListener!.start();
              _state = AudiopcState.playing;
              return AudiopcState.playing;
            }
          case 4.0:
            {
              _positionListener!.pause();
              _samplesListener!.pause();
              _state = AudiopcState.paused;
              return AudiopcState.paused;
            }
          case 5.0:
            {
              _state = AudiopcState.stopped;
              return AudiopcState.stopped;
            }
          default:
            return AudiopcState.none;
        }
      });

  Stream<bool> get onCompleted => Stream.value(isCompleted);

  Audiopc({required this.id}) {
    _platform.listen(id);

    _eventSubscription = _platform.eventStream[id].listen((event) {
      _eventStreamController.add(event);
    });
    _platform.init(id);

    _positionListener = PositionListener(
      getPosition: getPosition,
    );

    _samplesListener = SamplesListener(
      getSamples: getSamples,
    );
  }

  Future<void> play(String path) async {
    await _platform.setSource(path, id);
    await _platform.play(id);
  }

  Future<void> resume() async {
    await _platform.play(id);
  }

  Future<void> pause() async {
    await _platform.pause(id);
  }

  Future<void> seek(double position) async {
    await _platform.seek(position, id);
  }

  Future<void> setVolume() async {
    await _platform.setVolume(id);
  }

  Future<void> setRate(double rate) async {
    await _platform.setRate(rate, id);
  }

  Future<double> getPosition() async {
    return await _platform.getPosition(id) ?? 0.0;
  }

  Future<List<double>> getSamples() async {
    return await _platform.getSamples(id) ?? [];
  }

  Future<AudioMetaData> getMetadata(String path) async {
    final metadata = await _platform.getMetadata(path) ?? {};
    final temp = metadata.map((key, value) => MapEntry(key as String, value));
    return AudioMetaData.fromMap(temp);
  }

  /// Dispose the player
  ///
  /// This method should be called when the player is no longer needed.
  /// Throws an [Exception] if trying to call any method after calling this method
  void dispose() {
    _positionListener!.stop();
    _samplesListener!.stop();
    _eventSubscription!.cancel();
    _eventStreamController.close();
  }
}