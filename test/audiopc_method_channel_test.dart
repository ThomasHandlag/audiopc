import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:audiopc/audiopc_method_channel.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  MethodChannelAudiopc platform = MethodChannelAudiopc();
  const MethodChannel channel = MethodChannel('audiopc');

  setUp(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
      channel,
      (MethodCall methodCall) async {
        return '42';
      },
    );
  });

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(channel, null);
  });

  test('getPlatformVersion', () async {
    
  });
}
