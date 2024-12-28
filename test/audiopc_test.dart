import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:audiopc/audiopc.dart';
import 'package:audiopc/audiopc_platform_interface.dart';
import 'package:audiopc/audiopc_method_channel.dart';
import 'package:plugin_platform_interface/plugin_platform_interface.dart';

class MockAudiopcPlatform
    with MockPlatformInterfaceMixin
    implements AudiopcPlatform {
  @override
  Future<double?> getDuration() => Future.value(0.0);

  @override
  Future<double?> getPosition() => Future.value(0.0);

  @override
  Future<Float64List?> getSamples() => Future.value(Float64List(0));

  @override
  Future<double?> getState() => Future.value(0.0);

  @override
  Future<double?> getVolumn() => Future.value(0.0);

  @override
  Future<bool?> pause() => Future.value(true);

  @override
  Future<bool?> play() => Future.value(true);

  @override
  Future<double?> seek(double position) => Future.value(0.0);

  @override
  Future<bool?> setRate(double rate) => Future.value(true);

  @override
  Future<String?> setSource(String path) => Future.value('path');

  @override
  Future<bool?> setVolume() => Future.value(true);

}

void main() {
  final AudiopcPlatform initialPlatform = AudiopcPlatform.instance;

  test('$MethodChannelAudiopc is the default instance', () {
    expect(initialPlatform, isInstanceOf<MethodChannelAudiopc>());
  });

  test('getPlatformVersion', () async {
    Audiopc audiopcPlugin = Audiopc();
    MockAudiopcPlatform fakePlatform = MockAudiopcPlatform();
    AudiopcPlatform.instance = fakePlatform;

  });
}
