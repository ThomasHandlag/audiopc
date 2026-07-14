import 'dart:typed_data';

import 'package:test/test.dart';
import 'package:audiopc/audiopc.dart';

const assetPath = "test/assets/";

/// Placeholder test entrypoint for the audiopc package.
void main() {
  test("Device capabilities", () {
    final player = AudioPlayer();
    final backendInfo = player.getAudioBackendInfo();

    expect(
      backendInfo.outputDeviceCount,
      greaterThan(0),
      reason: "Audio backend should report usable output capabilities",
    );
  });

  group("Test playback features", () {
    final player = AudioPlayer();

    Uint8List testAudioData = Uint8List.fromList(
      List.generate(44100 * 2, (i) => (i % 256).toUnsigned(8)),
    );

    test("Set audio data", () {
      final ok = player.playMemory(testAudioData);
      expect(
        ok,
        isTrue,
        reason: "playMemory should return true for valid audio data",
      );
    });

    test("Seek within audio data", () {
      final ok = player.seek(1000); // Seek to 1 second
      expect(ok, isTrue, reason: "seek should return true for valid position");
    });

    test("Get thumbnail from file", () {
      final thumbnail = player.getThumbnail("test/assets/test_audio.mp3");
      expect(
        thumbnail,
        isNotNull,
        reason: "getThumbnail should return non-null for valid audio file",
      );
    }, skip: true); // Skipping this test for now since it requires an actual audio file
  });
}
