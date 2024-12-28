import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

import 'audiopc_platform_interface.dart';

/// An implementation of [AudiopcPlatform] that uses method channels.
class MethodChannelAudiopc extends AudiopcPlatform {
  /// The method channel used to interact with the native platform.
  @visibleForTesting
  final methodChannel = const MethodChannel('audiopc');

  @override
  Future<String?> setSource(String path) async {
    final result =
        await methodChannel.invokeMethod<String>('setSource', {"path": path});
    return result;
  }

  @override
  Future<bool?> play() async {
    final result = await methodChannel.invokeMethod<bool>('play');
    return result;
  }

  @override
  Future<bool?> pause() async {
    final result = await methodChannel.invokeMethod<bool>('pause');
    return result;
  }

  @override
  Future<double?> getDuration() async {
    final result = await methodChannel.invokeMethod<double>('getDuration');
    return result;
  }

  @override
  Future<Float64List?> getSamples() async {
    final result = await methodChannel.invokeMethod<Float64List?>('getSamples');
    return result;
  }

  @override
  Future<double?> seek(double position) async {
    final result = await methodChannel
        .invokeMethod<double>('seek', {"position": position});
    return result;
  }

  @override
  Future<bool?> setVolume() async {
    final result = await methodChannel.invokeMethod<bool>('setVolume');
    return result;
  }

  @override
  Future<double?> getVolumn() async {
    final result = await methodChannel.invokeMethod<double>('getVolume');
    return result;
  }

  @override
  Future<bool?> setRate(double rate) async {
    final result =
        await methodChannel.invokeMethod<bool>('setRate', {"rate": rate});
    return result;
  }

  @override
  Future<double?> getPosition() async {
    final result = await methodChannel.invokeMethod<double>('getPosition');
    return result;
  }

  @override
  Future<double?> getState() async {
    final result = await methodChannel.invokeMethod<double?>('getState');
    return result;
  }
}