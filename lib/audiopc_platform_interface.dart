import 'dart:typed_data';

import 'package:audiopc/audiopc_platform.dart';
import 'package:plugin_platform_interface/plugin_platform_interface.dart';

abstract class AudiopcPlatformInterface extends PlatformInterface
    implements AudioMethodChannelInterface {
  AudiopcPlatformInterface() : super(token: _token);

  static final Object _token = Object();

  /// The default instance of [AudiopcPlatformInterface] to use.
  ///
  /// Defaults to [AudiopcPlatform].
  static AudiopcPlatformInterface _instance = AudiopcPlatform();

  static AudiopcPlatformInterface get instance => _instance;

  /// Platform-specific implementations should set this with their own
  /// platform-specific class that extends [AudiopcPlatformInterface] when
  /// they register themselves.
  static set instance(AudiopcPlatformInterface instance) {
    PlatformInterface.verifyToken(instance, _token);
    _instance = instance;
  }
}

abstract class AudioMethodChannelInterface {

  Future<void> setVolume() {
    throw UnimplementedError('setVolume() has not been implemented.');
  }

  Future<void> setSource(String path) {
    throw UnimplementedError(
        'setSource(String path) has not been implemented.');
  }

  Future<void> play() {
    throw UnimplementedError('play() has not been implemented.');
  }

  Future<void> pause() {
    throw UnimplementedError('pause() has not been implemented.');
  }

  Future<Float64List?> getSamples() {
    throw UnimplementedError('getVolume() has not been implemented.');
  }

  Future<void> seek(double position) {
    throw UnimplementedError('seek(double position) has not been implemented.');
  }

  Future<double?> getPosition() {
    throw UnimplementedError('getPosition() has not been implemented.');
  }

  Future<void> setRate(double rate) {
    throw UnimplementedError('setRate(double rate) has not been implemented.');
  }
}

abstract class AudioEventChannelInterface {
  void listen() {
    throw UnimplementedError('listen() has not been implemented.');
  }
}
