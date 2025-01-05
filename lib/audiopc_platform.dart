import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'audiopc_platform_interface.dart';

/// An implementation of [AudiopcPlatform] that uses method channels.
class AudiopcPlatform extends AudiopcPlatformInterface with AudioEventChannel {
  // / The method channel used to interact with the native platform.
  @visibleForTesting
  final methodChannel = const MethodChannel('audiopc/methodChannel'); 

  void create() {
    listen();
  }

  @override
  Future<void> setSource(String path) async {
    return await _call('setSource', {"path": path});
  }

  @override
  Future<void> play() async {
    return await _call('play', {});
  }

  @override
  Future<void> pause() async {
    return await _call('pause', {});
  }

  @override
  Future<Float64List?> getSamples() async {
    return await _listen('getSamples');
  }

  @override
  Future<void> seek(double position) async {
    return await _call('seek', {"position": position});
  }

  @override
  Future<void> setVolume() async {
    return await _call('setVolume', {});
  }

  @override
  Future<void> setRate(double rate) async {
    return await _call('setRate', {"rate": rate});
  }

  @override
  Future<double?> getPosition() async {
    return await _listen('getPosition');
  }

  Future<void> _call(String method, Map<String, dynamic> params) {
    return methodChannel.invokeMethod(method, params);
  }

  Future<T?> _listen<T>(String method) {
    return methodChannel.invokeMethod<T>(method);
  }
}

mixin AudioEventChannel implements AudioEventChannelInterface {

  final eventChannel = const EventChannel('audiopc/eventChannel');
  static Stream<dynamic> _eventStream = const Stream.empty();

  get eventStream => _eventStream;

  @override
  void listen() {
    _eventStream = eventChannel
        .receiveBroadcastStream()
        .map((event) {
          final eventName = event['event'] as String;

          switch (eventName) {
            case 'duration':
              {
                final duration = event['value'] as double;
                return duration;
              }
            case 'state':
              {
                final position = event['value'] as double;
                return position;
              }
            case 'completed':
              {
                return true;
              }
          }
        });
  }
}
