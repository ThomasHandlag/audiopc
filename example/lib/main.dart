import 'package:flutter/material.dart';
import 'package:file_picker/file_picker.dart';
import 'package:audiopc/audiopc.dart' as audiopc;

void main() {
  runApp(const MyApp());
}

class MyApp extends StatefulWidget {
  const MyApp({super.key});

  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  final player = audiopc.AudiopcPlayer();
  final sourceController = TextEditingController();
  final volumeController = TextEditingController(text: '1.0');
  final lowPassController = TextEditingController(text: '0');
  final backendInfo = audiopc.getAudioBackendInfo();

  String status = 'Idle';
  String bufferedSamples = '0';
  String positionMillis = '0';
  String durationMillis = '-1';
  bool isUrlSource = false;
  double _sliderPosition = 0;

  @override
  void dispose() {
    sourceController.dispose();
    volumeController.dispose();
    lowPassController.dispose();
    player.stop();
    super.dispose();
  }

  void updateBufferedSamples() {
    setState(() {
      bufferedSamples = player.bufferedSamples.toString();
      positionMillis = player.positionMillis.toString();
      durationMillis = player.durationMillis.toString();
      final pos = int.tryParse(positionMillis) ?? 0;
      _sliderPosition = pos.toDouble();
    });
  }

  void seekToMillis(double ms) {
    final target = ms.toInt();
    player.seek(target);
    setState(() {
      _sliderPosition = ms;
      status = 'Seek requested: $target ms';
    });
    updateBufferedSamples();
  }

  void loadSource() {
    final source = sourceController.text.trim();
    if (source.isEmpty) {
      setState(() {
        status = 'Enter a file path or URL.';
      });
      return;
    }

    final success = isUrlSource
        ? player.setUrlSource(source)
        : player.setFileSource(source);

    setState(() {
      status = success
          ? 'Source loaded.'
          : 'Failed to load source';
    });
    updateBufferedSamples();
  }

  void play() {
    final success = player.play();
    setState(() {
      status = success
          ? 'Playing.'
          : 'Play failed';
    });
    updateBufferedSamples();
  }

  void pause() {
    final success = player.pause();
    setState(() {
      status = success
          ? 'Paused.'
          : 'Pause failed';
    });
  }

  void stop() {
    final success = player.stop();
    setState(() {
      status = success
          ? 'Stopped.'
          : 'Stop failed';
      bufferedSamples = '0';
    });
  }

  void applyProcessing() {
    final volume = double.tryParse(volumeController.text.trim()) ?? 1.0;
    final lowPass = double.tryParse(lowPassController.text.trim()) ?? 0.0;

    final volumeOk = player.setVolume(volume);
    final lowPassOk = player.setLowPassHz(lowPass);

    setState(() {
      status = volumeOk && lowPassOk
          ? 'Processing updated.'
          : 'Failed to update processing';
    });
  }

  void _selectFile() async {
    final result = await FilePicker.pickFiles(type: FileType.audio);

    if (result != null && result.files.single.path != null) {
      sourceController.text = result.files.single.path!;
    }
  }

  String _formatTime(int ms) {
    final seconds = ms ~/ 1000;
    final minutes = seconds ~/ 60;
    final secs = seconds % 60;
    return '${minutes.toString().padLeft(2, '0')}:${secs.toString().padLeft(2, '0')}';
  }

  @override
  initState() {
    super.initState();
    player.positionStreamController.stream.listen((pos) {
      setState(() {
        positionMillis = pos.toString();
        _sliderPosition = pos.toDouble();
      });
    });
  }

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;

    return MaterialApp(
      debugShowCheckedModeBanner: false,
      home: Scaffold(
        backgroundColor: scheme.surface,
        appBar: AppBar(title: const Text('audiopc demo')),
        body: SafeArea(
          child: ListView(
            padding: const EdgeInsets.all(20),
              children: [
              Text(
                'CPAL backend ready: ${backendInfo.isAvailable}',
                style: Theme.of(context).textTheme.titleMedium,
                ),
              const SizedBox(height: 8),
                Text(
                'Output sample rate: ${backendInfo.defaultOutputSampleRate}',
              ),
              Text('Output channels: ${backendInfo.defaultOutputChannels}'),
              Text('Output device count: ${backendInfo.outputDeviceCount}'),
              const SizedBox(height: 24),
              SegmentedButton<bool>(
                segments: const [
                  ButtonSegment(value: false, label: Text('Local file')),
                  ButtonSegment(value: true, label: Text('URL stream')),
                ],
                selected: {isUrlSource},
                onSelectionChanged: (selected) {
                  setState(() {
                    isUrlSource = selected.first;
                  });
                },
              ),
              const SizedBox(height: 12),
              isUrlSource
                  ? TextField(
                      controller: sourceController,
                      decoration: InputDecoration(
                        border: const OutlineInputBorder(),
                        labelText: 'Audio URL',
                      ),
                    )
                  : ElevatedButton(
                      onPressed: _selectFile,
                      child: const Text("Select a file"),
                    ),
              const SizedBox(height: 12),
              Wrap(
                spacing: 12,
                runSpacing: 12,
                children: [
                  FilledButton(
                    onPressed: loadSource,
                    child: const Text('Load'),
                  ),
                  FilledButton(onPressed: play, child: const Text('Play')),
                  OutlinedButton(onPressed: pause, child: const Text('Pause')),
                  OutlinedButton(onPressed: stop, child: const Text('Stop')),
                ],
              ),
              const SizedBox(height: 20),
              TextField(
                controller: volumeController,
                keyboardType: TextInputType.number,
                decoration: const InputDecoration(
                  border: OutlineInputBorder(),
                  labelText: 'Volume',
                  helperText: '1.0 = normal, 0.5 = half, 2.0 = boost',
                ),
              ),
              const SizedBox(height: 12),
              TextField(
                controller: lowPassController,
                keyboardType: TextInputType.number,
                decoration: const InputDecoration(
                  border: OutlineInputBorder(),
                  labelText: 'Low-pass cutoff (Hz)',
                  helperText: '0 disables filtering',
                ),
              ),
              const SizedBox(height: 12),
              FilledButton(
                onPressed: applyProcessing,
                child: const Text('Apply processing'),
              ),
              const SizedBox(height: 20),
              Text(
                'Seek position',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 12),
              Row(
                spacing: 12,
                children: [
                  Text(_formatTime(int.tryParse(positionMillis) ?? 0)),
                  Expanded(
                    child: Slider(
                      value: _sliderPosition,
                      min: 0,
                      max: (int.tryParse(durationMillis) ?? 0).toDouble().clamp(0, double.maxFinite),
                      onChanged: (value) {
                        setState(() {
                          _sliderPosition = value;
                        });
                      },
                      onChangeEnd: (value) {
                        seekToMillis(value);
                      },
                    ),
                  ),
                  Text(_formatTime(int.tryParse(durationMillis) ?? 0)),
                ],
              ),
              const SizedBox(height: 20),
              FilledButton.tonal(
                onPressed: updateBufferedSamples,
                child: const Text('Refresh buffered samples'),
              ),
              const SizedBox(height: 12),
              Text('Buffered samples: $bufferedSamples'),
              Text('Position (ms): $positionMillis'),
              Text('Duration (ms): $durationMillis'),
              const SizedBox(height: 24),
              Text(
                'Status: $status',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 12),
              const Text(
                'Supported formats are handled by Symphonia in the Rust backend.\n'
                'For internet playback, provide a direct media URL.',
                ),
              ],
            ),
          ),
        ),
    );
  }
}
