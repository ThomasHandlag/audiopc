import 'dart:async';

import 'package:audiopc/state.dart';
import 'package:file_picker/file_picker.dart';
import 'package:audiopc/audiopc.dart' as audiopc;
import 'package:flutter/material.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatefulWidget {
  const MyApp({super.key});

  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  static const int _visualizerFps = 60;
  static const int _spectrumBinCount = 64;

  final audiopc.AudiopcPlayer player = audiopc.AudiopcPlayer();
  final sourceController = TextEditingController();
  final lowPassController = TextEditingController(text: '0');
  final backendInfo = audiopc.getAudioBackendInfo();

  late final StreamSubscription _positionSubscription;
  late final Timer _visualizerTimer;

  String bufferedSamples = '0';
  String positionMillis = '0';
  String durationMillis = '-1';
  bool isUrlSource = false;
  double _sliderPosition = 0;
  double _volumePercent = 100;
  List<double> _spectrumBars = List<double>.filled(_spectrumBinCount, 0);

  @override
  void dispose() {
    _positionSubscription.cancel();
    _visualizerTimer.cancel();
    sourceController.dispose();
    lowPassController.dispose();
    player.dispose();
    super.dispose();
  }

  double _volumeFromSlider(double sliderValue) {
    return 0.1 + (sliderValue.clamp(0, 100) / 100) * 0.9;
  }

  String _volumeLabel(double sliderValue) {
    return _volumeFromSlider(sliderValue).toStringAsFixed(2);
  }

  void _syncPlayerState() {
    bufferedSamples = player.bufferedSamples.toString();
    positionMillis = player.positionMillis.toString();
    durationMillis = player.durationMillis.toString();
    final pos = int.tryParse(positionMillis) ?? 0;
    _sliderPosition = pos.toDouble();
  }

  void updateBufferedSamples() {
    setState(() {
      _syncPlayerState();
    });
  }

  void seekToMillis(double ms) {
    final target = ms.toInt();
    player.seek(target);
    setState(() {
      _sliderPosition = ms;
    });
    updateBufferedSamples();
  }

  void loadSource() {
    final source = sourceController.text.trim();
    if (source.isEmpty) {
      setState(() {});
      return;
    }

    isUrlSource ? player.setUrlSource(source) : player.setFileSource(source);

    setState(() {});
    updateBufferedSamples();
  }

  void play() {
    player.play();
    updateBufferedSamples();
  }

  void pause() {
    player.pause();
  }

  void stop() {
    player.stop();
    setState(() {
      bufferedSamples = '0';
    });
  }

  void setVolume(double sliderValue) {
    setState(() {
      _volumePercent = sliderValue;
    });
    player.setVolume(_volumeFromSlider(sliderValue));
  }

  void _updateSpectrum() {
    final next = player.getVisualizerSamples(_spectrumBinCount);
    if (next.isEmpty) {
      return;
    }

    if (!mounted) {
      return;
    }

    setState(() {
      _spectrumBars = next;
    });
  }

  void applyProcessing() {
    final lowPass = double.tryParse(lowPassController.text.trim()) ?? 0.0;

    player.setLowPassHz(lowPass);
  }

  Future<void> _selectFile() async {
    final result = await FilePicker.pickFiles(type: FileType.audio);

    final path = result?.files.single.path;
    if (path != null) {
      sourceController.text = path;
    }
  }

  String _formatTime(int ms) {
    final seconds = ms ~/ 1000;
    final minutes = seconds ~/ 60;
    final secs = seconds % 60;
    return '${minutes.toString().padLeft(2, '0')}:${secs.toString().padLeft(2, '0')}';
  }

  @override
  void initState() {
    super.initState();
    _positionSubscription = player.positionStreamController.stream.listen((
      pos,
    ) {
      setState(() {
        positionMillis = pos.toString();
        _sliderPosition = pos.toDouble();
      });
    });
    _syncPlayerState();
    _visualizerTimer = Timer.periodic(
      const Duration(milliseconds: 100 ~/ _visualizerFps),
      (_) => _updateSpectrum(),
    );
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
                      decoration: const InputDecoration(
                        border: OutlineInputBorder(),
                        labelText: 'Audio URL',
                      ),
                    )
                  : ElevatedButton(
                      onPressed: _selectFile,
                      child: const Text('Select a file'),
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
                  StreamBuilder(
                    stream: player.stateStream,
                    builder: (context, snapshot) {
                      final state = snapshot.data ?? PlayerState.error;
                      return OutlinedButton(
                        onPressed: () {
                          switch (state) {
                            case PlayerState.playing:
                              player.pause();
                              break;
                            case PlayerState.paused:
                              player.play();
                              break;
                            case PlayerState.idle:
                              player.play();
                              break;
                            case PlayerState.loading:
                              break;
                            case _:
                              break;
                          }
                        },
                        child: Text(
                          state == PlayerState.playing ? 'Pause' : 'Play',
                        ),
                      );
                    },
                  ),
                  OutlinedButton(onPressed: stop, child: const Text('Stop')),
                ],
              ),
              const SizedBox(height: 20),
              Text(
                'Volume: ${_volumeLabel(_volumePercent)}',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 8),
              Slider(
                value: _volumePercent,
                min: 0,
                max: 100,
                divisions: 100,
                label: _volumeLabel(_volumePercent),
                onChanged: setVolume,
              ),
              const SizedBox(height: 4),
              Text(
                'Slider range 0 to 100 maps to volume 0.10 to 1.00.',
                style: Theme.of(context).textTheme.bodySmall,
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
                'Spectrum visualizer',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 8),
              SizedBox(
                height: 320,
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.end,
                  children: [
                    for (final value in _spectrumBars)
                      Expanded(
                        child: Padding(
                          padding: const EdgeInsets.symmetric(horizontal: 2),
                          child: AnimatedContainer(
                            duration: const Duration(milliseconds: 90),
                            curve: Curves.easeOut,
                            height: 8 + (value * 112).abs(),
                            decoration: BoxDecoration(
                              color: Color.lerp(
                                Colors.greenAccent,
                                Colors.deepOrange,
                                value,
                              ),
                              borderRadius: BorderRadius.circular(4),
                            ),
                          ),
                        ),
                      ),
                  ],
                ),
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
                      value: _sliderPosition.clamp(
                        0,
                        (int.tryParse(durationMillis) ?? 0).toDouble().clamp(
                          0,
                          double.maxFinite,
                        ),
                      ),
                      min: 0,
                      max: (int.tryParse(durationMillis) ?? 0).toDouble().clamp(
                        0,
                        double.maxFinite,
                      ),
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
