import 'dart:async';

import 'package:audiopc/audiopc_platform.dart';
import 'package:audiopc/audiopc_state.dart';
import 'package:audiopc/audopc_helper.dart';
import 'package:audiopc/audio_metadata.dart';
import 'package:audiopc/player_event.dart';
import 'package:flutter/material.dart';

part 'widgets/audiopc_slider.dart';

class Audiopc {
  final _platform = AudiopcPlatform();
  final String id;

  double _duration = 0.0;
  double get duration => _duration;

  PlayerState _state = PlayerState.none;
  PlayerState get state => _state;

  bool get isPlaying => _state == PlayerState.playing;
  bool get isPaused => _state == PlayerState.paused;

  PositionListener? _positionListener;

  Stream<double> get onPositionChanged =>
      _positionListener!.streamControler.stream;

  StreamSubscription<dynamic>? _eventSubscription;

  final _eventStreamController = StreamController<PlayerEvent>.broadcast();

  Stream<String> get onError => _eventStreamController.stream
      .where((event) => event.type == PlayerEventType.error)
      .map((event) => event.value as String);

  Stream<PlayerState> get onStateChanged => _eventStreamController.stream
      .where((event) => event.type == PlayerEventType.state)
      .map((event) {
        final stateValue = event.value as double;
        switch (stateValue) {
          case 3.0:
            {
              return PlayerState.playing;
            }
          case 4.0:
            {
              return PlayerState.paused;
            }
          case 5.0:
            {
              return PlayerState.stopped;
            }
          default:
            {
              return PlayerState.none;
            }
        }
      });

  Stream<List<double>> get onSamples => _eventStreamController.stream
      .where((event) => event.type == PlayerEventType.samples)
      .map((event) {
        return event.value as List<double>;
      });

  Stream<double> get onDurationChanged => _eventStreamController.stream
      .where((event) => event.type == PlayerEventType.duration)
      .map((event) {
        return event.value as double;
      });

  Stream<bool> get onCompleted => _eventStreamController.stream
      .where((event) => event.type == PlayerEventType.completed)
      .map((event) {
        return event.value as bool;
      });

  Audiopc({required this.id}) {
    _platform.listen(id);

    _eventSubscription = _platform.eventStream[id].listen((event) {
      _eventStreamController.add(event);
    });
    _platform.init(id);

    _positionListener = PositionListener(getPosition: getPosition);
    onDurationChanged.listen((duration) {
      _duration = duration;
    });

    onStateChanged.listen((state) {
      _state = state;
      if (state == PlayerState.playing) {
        _positionListener!.start();
      }
    });

    onError.listen((error) {
      debugPrint("Audiopc Error: $error");
    });
  }

  Future<void> play(String path) async {
    await _platform.setSource(path, id);
    await _platform.play(id);
    _positionListener!.start();
  }

  Future<void> resume() async {
    await _platform.play(id);
    _positionListener!.start();
  }

  Future<void> pause() async {
    await _platform.pause(id);
    _positionListener!.pause();
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
    _eventSubscription!.cancel();
    _eventStreamController.close();
    _platform.close(id);
  }
}
