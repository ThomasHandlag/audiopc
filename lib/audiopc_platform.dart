import 'package:audiopc/player_event.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'audiopc_platform_interface.dart';

/// An implementation of [AudiopcPlatform] that uses method channels.
class AudiopcPlatform extends AudiopcPlatformInterface with AudioEventChannel {
  // / The method channel used to interact with the native platform.
  @visibleForTesting
  final methodChannel = const MethodChannel('audiopc/methodChannel');

  /// Initializes the platform interface and create a new instance of the player.
  @override
  Future<void> init(String id) {
    return _call('init', {"id": id}, id);
  }

  /// Sets the source of the player.
  @override
  Future<void> setSource(String path, String id) async {
    return await _call('setSource', {"path": path}, id);
  }

  @override
  Future<void> play(String id) async {
    return await _call('play', {}, id);
  }

  @override
  Future<void> pause(String id) async {
    return await _call('pause', {}, id);
  }

  @override
  Future<void> seek(double position, String id) async {
    return await _call('seek', {"position": position}, id);
  }

  @override
  Future<void> setVolume(String id) async {
    return await _call('setVolume', {}, id);
  }

  @override
  Future<void> setRate(double rate, String id) async {
    return await _call('setRate', {"rate": rate}, id);
  }

  @override
  Future<double?> getPosition(String id) async {
    return await _listen('getPosition', id);
  }

  @override
  Future<void> close(String id) async {
    return await _call('close', {}, id);
  }

  @override
  Future<Map<dynamic, dynamic>?> getMetadata(String path) async {
    return await methodChannel.invokeMethod('getMetaData', {"path": path});
  }

  Future<void> _call(String method, Map<String, dynamic> params, String id) {
    return methodChannel.invokeMethod(
        method, params..addEntries([MapEntry('id', id)]));
  }

  Future<T?> _listen<T>(String method, String id) {
    return methodChannel.invokeMethod<T>(method, {"id": id});
  }
}

mixin AudioEventChannel implements AudioEventChannelInterface {
  final eventChannel = const EventChannel('audiopc/eventChannel');
  static final Map<String, Stream<dynamic>> _eventStream = {};

  get eventStream => _eventStream;

  @override
  void listen(String id) {
    _eventStream[id] = eventChannel.receiveBroadcastStream().map((event) {
      if (event['id'] == id) {
        final eventName = event['event'] as String;
        switch (eventName) {
          case 'state':
            {
              final state = event['value'] as int;
              return StateEvent(value: state.toDouble());
            }
          case 'error':
            {
              return ErrorEvent(value: event['value'] as String);
            }
          case 'duration':
            {
              return DurationEvent(value: event['value'] as double);
            }
          case 'samples':
            {
              final samples = (event['value'] as List<dynamic>)
                  .map((e) => e as double)
                  .toList();
              return SamplesEvent(value: samples);
            }
          case 'completed':
            {
              return CompletedEvent(value: event['value'] as bool);
            }
          default:
            {
              return NoneEvent();
            }
        }
      }
    });
  }
}


