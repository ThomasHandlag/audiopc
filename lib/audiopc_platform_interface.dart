import 'dart:typed_data';

import 'package:plugin_platform_interface/plugin_platform_interface.dart';

import 'audiopc_method_channel.dart';

abstract class AudiopcPlatform extends PlatformInterface {
  /// Constructs a AudiopcPlatform.
  AudiopcPlatform() : super(token: _token);

  static final Object _token = Object();

  static AudiopcPlatform _instance = MethodChannelAudiopc();

  /// The default instance of [AudiopcPlatform] to use.
  ///
  /// Defaults to [MethodChannelAudiopc].
  static AudiopcPlatform get instance => _instance;

  /// Platform-specific implementations should set this with their own
  /// platform-specific class that extends [AudiopcPlatform] when
  /// they register themselves.
  static set instance(AudiopcPlatform instance) {
    PlatformInterface.verifyToken(instance, _token);
    _instance = instance;
  }

  Future<bool?> setVolume() {
    throw UnimplementedError('setVolume() has not been implemented.');
  }

  Future<double?> getVolumn() {
    throw UnimplementedError('getVolume() has not been implemented.');
  }

  Future<String?> setSource(String path) {
    throw UnimplementedError(
        'setSource(String path) has not been implemented.');
  }

  Future<bool?> play() {
    throw UnimplementedError('play() has not been implemented.');
  }

  Future<bool?> pause() {
    throw UnimplementedError('pause() has not been implemented.');
  }

  Future<double?> getDuration() {
    throw UnimplementedError('getDuration() has not been implemented.');
  }

  Future<Float64List?> getSamples() {
    throw UnimplementedError('getVolume() has not been implemented.');
  }

  Future<double?> seek(double position) {
    throw UnimplementedError('seek(double position) has not been implemented.');
  }

  Future<double?> getPosition() {
    throw UnimplementedError('getPosition() has not been implemented.');
  }

  Future<bool?> setRate(double rate) {
    throw UnimplementedError('setRate(double rate) has not been implemented.');
  }

  Future<double?> getState() {
    throw UnimplementedError('getState() has not been implemented.');
  }
}