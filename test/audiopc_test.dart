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
  Future<bool?> setVolume() => Future.value(false);

  @override
  Future<String?> setSource(String path) => Future.value("");

  @override
  Future<bool?> play() => Future.value(false);

  @override
  Future<bool?> pause() => Future.value(true);

  @override
  Future<double?> getDuration() => Future.value(0.0);

  @override
  Future<double?> getPosition() => Future.value(0.0);

  @override
  Future<double?> seek(double position) => Future.value(0);

  @override
  Future<bool?> setRate(double rate) => Future.value(true);

  @override
  Future<double?> getVolumn() => Future.value(0.0);

  @override
  Future<double?> getState() => Future.value(0);
  
  @override
  Future<Float64List?> getSamples() => Future.value(Float64List(0));
}

void main() {
  final AudiopcPlatform initialPlatform = AudiopcPlatform.instance;

  test('$MethodChannelAudiopc is the default instance', () {
    expect(initialPlatform, isInstanceOf<MethodChannelAudiopc>());
  });

  test('setVolume', () async {
    Audiopc audiopcPlug = Audiopc();
    MockAudiopcPlatform fakePlatform = MockAudiopcPlatform();
    AudiopcPlatform.instance = fakePlatform;
    AudiopcPlatform.instance = fakePlatform;

    expect(await audiopcPlug.setVolume(), true);
  });

  test('setSource', () async {
    Audiopc audiopcPlug = Audiopc();
    MockAudiopcPlatform fakePlatform = MockAudiopcPlatform();
    AudiopcPlatform.instance = fakePlatform;
    AudiopcPlatform.instance = fakePlatform;

    expect(await audiopcPlug.setSource("D:/Downloads/goagain.mp3"), true);
  });
}
