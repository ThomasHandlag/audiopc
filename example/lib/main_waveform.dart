import 'dart:async';
import 'package:audiopc/audiopc.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/foundation.dart';
import 'package:material_ui/material_ui.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();

  runApp(const AudioDemoApp());
}

class AudioDemoApp extends StatelessWidget {
  const AudioDemoApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      theme: ThemeData(colorSchemeSeed: Colors.deepPurple, useMaterial3: true),
      home: const AudioDemoPage(),
    );
  }
}

class AudioDemoPage extends StatefulWidget {
  const AudioDemoPage({super.key});

  @override
  State<AudioDemoPage> createState() => _AudioDemoPageState();
}

class _AudioDemoPageState extends State<AudioDemoPage> {
  late final PcPlayer _player;

  Timer? _timer;
  StreamSubscription<int>? _positionSubscription;

  Float32List _waveformData = Float32List(0);
  double _position = 0;
  int _duration = 0;
  bool _isPlaying = false;

  String? audioSource;

  @override
  void initState() {
    super.initState();
    _player = PcPlayer.instance();
    _positionSubscription = _player.positionStream.listen((position) {
      if (!mounted) return;

      setState(() {
        _position = position.toDouble();
        _duration = _player.duration;
      });
    });
  }

  void getWaveform(String source) async {
    final data = await getAudioWaveform(
      numPoints: BigInt.from(50),
      path: source,
    );

    if (!mounted || data.isEmpty) return;

    setState(() {
      _waveformData = data;
    });
  }

  Future<void> _loadAudio(String path) async {
    _player.setSource(AudioSource.path(path));
    setState(() {
      _position = 0;
      _isPlaying = false;
    });
  }

  void _togglePlayback() {
    if (_isPlaying) {
      _player.pause();
    } else {
      _player.play();
    }

    setState(() {
      _isPlaying = !_isPlaying;
    });
  }

  void _seek(double value) {
    _player.seek(value.toInt());

    setState(() {
      _position = value;
    });
  }

  @override
  void dispose() {
    _timer?.cancel();
    _positionSubscription?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final duration = _duration > 0 ? _duration.toDouble() : 1.0;
    final position = _position.clamp(0.0, duration);

    return Scaffold(
      appBar: AppBar(title: const Text('AudioPC Waveform Demo')),
      body: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            WaveformControl(
              value: position / duration,
              data: _waveformData,
              onChanged: (value) {
                _seek(value * duration);
              },
            ),
            const SizedBox(height: 24),
            Text(
              '${_formatTime(_position.toInt())} / '
              '${_formatTime(_duration)}',
            ),
            const SizedBox(height: 16),
            FilledButton.icon(
              onPressed: _togglePlayback,
              icon: Icon(_isPlaying ? Icons.pause : Icons.play_arrow),
              label: Text(_isPlaying ? 'Pause' : 'Play'),
            ),
            const SizedBox(height: 12),
            OutlinedButton(
              onPressed: () async {
                final result = await FilePicker.pickFiles(type: FileType.audio);

                final path = result?.files.single.path;
                if (path != null) {
                  setState(() {
                    audioSource = path;
                  });
                  getWaveform(audioSource!);
                }
              },
              child: const Text('Select audio'),
            ),
            TextButton(
              onPressed: () async {
                if (audioSource == null) {
                  return;
                }
                _loadAudio(audioSource!);
              },
              child: Text('Play Source'),
            ),
          ],
        ),
      ),
    );
  }

  String _formatTime(int milliseconds) {
    final seconds = milliseconds ~/ 1000;
    final minutes = seconds ~/ 60;
    return '$minutes:${(seconds % 60).toString().padLeft(2, '0')}';
  }
}
