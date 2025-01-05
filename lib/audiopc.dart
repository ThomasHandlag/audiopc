import 'dart:async';

import 'package:audiopc/audiopc_platform.dart';
import 'package:audiopc/audiopc_state.dart';
import 'package:audiopc/audopc_helper.dart';
import 'package:audiopc/player_event.dart';

class Audiopc {
  final _platform = AudiopcPlatform();

  PositionListener? _positionListener;
  SamplesListener? _samplesListener;

  Stream<double> get onPositionChanged =>
      _positionListener!.streamControler.stream;
  Stream<List<double>> get onSamples =>
      _samplesListener!.streamControler.stream;

  StreamSubscription<PlayerEvent>? _eventSubscription;

  final _eventStreamController = StreamController<PlayerEvent>.broadcast();

  Stream<double> get onDurationChanged => _eventStreamController.stream
      .where((event) => event.type == PlayerEventType.duration)
      .map((event) => event.value as double);

  Stream<AudiopcState> get onStateChanged => _eventStreamController.stream
          .where((event) => event.type == PlayerEventType.position)
          .map((event) {
        final state = event.value as double;
        switch (state) {
          case 3.0:
            return AudiopcState.playing;
          case 4.0:
            return AudiopcState.paused;
          case 5.0:
            return AudiopcState.stopped;
          default:
            return AudiopcState.none;
        }
      });

  Stream<bool> get onCompleted => _eventStreamController.stream
      .where((event) => event.type == PlayerEventType.completed)
      .map((event) => event.value as bool);

  Audiopc() {
    _eventSubscription = _platform.eventStream.listen((event) {
      _eventStreamController.add(event);
    });

    _platform.create();

    _positionListener = PositionListener(
      getPosition: getPosition,
      id: 0,
    );

    _samplesListener = SamplesListener(
      getSamples: getSamples,
      id: 1,
    );
  }

  Future<void> play(String path) async {
    await _platform.setSource(path);
    await _platform.play();
    _positionListener!.start();
    _samplesListener!.start();
  }

  Future<void> resume() async {
    await _platform.play();
    _positionListener!.start();
    _samplesListener!.start();
  }

  Future<void> pause() async {
    await _platform.pause();
    _positionListener!.pause();
    _samplesListener!.pause();
  }

  Future<void> seek(double position) async {
    await _platform.seek(position);
  }

  Future<void> setVolume() async {
    await _platform.setVolume();
  }

  Future<void> setRate(double rate) async {
    await _platform.setRate(rate);
  }

  Future<double> getPosition() async {
    return await _platform.getPosition() ?? 0.0;
  }

  Future<List<double>> getSamples() async {
    return await _platform.getSamples() ?? [];
  }

  void dispose() {
    _positionListener!.stop();
    _samplesListener!.stop();
    _eventSubscription!.cancel();
    _eventStreamController.close();
  }
}
