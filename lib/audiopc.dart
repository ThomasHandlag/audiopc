import 'dart:async';

import 'package:audiopc/audiopc_platform.dart';
import 'package:audiopc/audiopc_state.dart';
import 'package:audiopc/audopc_helper.dart';
import 'package:audiopc/player_event.dart';

class Audiopc {
  final _platform = AudiopcPlatform();
  final String id;

  double duration = 0.0;

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
          .where((event) => event.type == PlayerEventType.position)
          .map((event) {
        final state = event.value as double;
        switch (state) {
          case 3.0:
            {
              _positionListener!.start();
              _samplesListener!.start();
              return AudiopcState.playing;
            }
          case 4.0:
            {
              _positionListener!.pause();
              _samplesListener!.pause();
              return AudiopcState.paused;
            }
          case 5.0:
            {
              return AudiopcState.stopped;
            }
          default:
            return AudiopcState.none;
        }
      });

  Stream<bool> get onCompleted => _eventStreamController.stream
          .where((event) => event.type == PlayerEventType.completed)
          .map((event) {
        if (event.value) {
          _positionListener!.pause();
          _samplesListener!.pause();
        }
        return event.value as bool;
      });

  Audiopc({required this.id}) {
    _platform.listen(id);

    _eventSubscription = _platform.eventStream[id].listen((event) {
      _eventStreamController.add(event);
    });
    _platform.init(id);

    _positionListener = PositionListener(
      getPosition: getPosition,
      id: 0,
    );

    _samplesListener = SamplesListener(
      getSamples: getSamples,
      id: 0,
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

  void dispose() {
    _positionListener!.stop();
    _samplesListener!.stop();
    _eventSubscription!.cancel();
    _eventStreamController.close();
  }
}
