import 'dart:async';

import 'package:audiopc/audiopc.dart';
import 'package:audiopc_example/filters.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart' hide MetaData;

/// Example app entry point.
void main() {
  runApp(const MaterialApp(debugShowCheckedModeBanner: false, home: MyApp()));
}

/// Demo app showing source loading, playback control, and visualization.
class MyApp extends StatefulWidget {
  const MyApp({super.key});

  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  static const int _visualizerFps = 60;
  static const int _spectrumBinCount = 64;

  final AudioPlayer player = AudioPlayer();
  final sourceController = TextEditingController();
  final lowPassController = TextEditingController(text: '0');
  late final backendInfo = player.getAudioBackendInfo();

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
    _visualizerTimer.cancel();
    sourceController.dispose();
    lowPassController.dispose();
    player.dispose();
    super.dispose();
  }

  /// Converts slider percentage to backend volume scale.
  double _volumeFromSlider(double sliderValue) {
    return 0.1 + (sliderValue.clamp(0, 100) / 100) * 0.9;
  }

  /// Returns a formatted label for the current slider volume.
  String _volumeLabel(double sliderValue) {
    return _volumeFromSlider(sliderValue).toStringAsFixed(2);
  }

  /// Seeks playback to the given position in milliseconds.
  void seekToMillis(double ms) {
    final target = ms.toInt();
    player.seek(target);
    setState(() {
      _sliderPosition = ms;
    });
  }

  /// Loads the currently entered source.
  void loadSource() {
    final source = sourceController.text.trim();
    if (source.isEmpty) {
      setState(() {});
      return;
    }

    player.playSource(source);

    setState(() {});
  }

  /// Starts playback.
  void play() {
    player.play();
  }

  /// Pauses playback.
  void pause() {
    player.pause();
  }

  /// Stops playback and resets selected UI fields.
  void stop() {
    player.stop();
  }

  /// Applies slider volume to the audio backend.
  void setVolume(double sliderValue) {
    setState(() {
      _volumePercent = sliderValue;
    });
    player.setVolume(_volumeFromSlider(sliderValue));
  }

  /// Updates spectrum bars from the current visualizer frame.
  void _updateSpectrum() {
    final next = player.getVisualizerSpectrum(_spectrumBinCount);
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

  MetaData? _metadata;

  void _getMetadata() async {
    final url = sourceController.text.trim();
    if (url.isEmpty) {
      return;
    }

    final metadata = await player.getMetadata(url);
    setState(() {
      _metadata = metadata;
    });
  }

  void _getThumb() async {
    final url = sourceController.text.trim();
    if (url.isEmpty) {
      return;
    }

    final thumbData = await player.getThumbnail(url);
    if (thumbData == null) {
      // ignore: use_build_context_synchronously
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('No thumbnail available for this audio file'),
        ),
      );
      return;
    }
    setState(() {
      _thumbByte = thumbData;
    });
  }

  Uint8List? _thumbByte;

  /// Opens a file picker and fills the source field.
  Future<void> _selectFile() async {
    final result = await FilePicker.pickFiles(type: FileType.audio);  

    final path = result?.files.single.path;
    if (path != null) {
      sourceController.text = path;
    }
  }

  /// Formats milliseconds as mm:ss.
  String _formatTime(int ms) {
    final seconds = ms ~/ 1000;
    final minutes = seconds ~/ 60;
    final secs = seconds % 60;
    return '${minutes.toString().padLeft(2, '0')}:${secs.toString().padLeft(2, '0')}';
  }

  @override
  void initState() {
    super.initState();
    _visualizerTimer = Timer.periodic(
      const Duration(milliseconds: 100 ~/ _visualizerFps),
      (_) => _updateSpectrum(),
    );
    player.positionStream.listen((pos) {
      setState(() {
        _sliderPosition = pos.toDouble();
      });
    });
  }

  double _rate = 1.0;

  String _rateLabel(double rate) => '${rate.toStringAsFixed(2)}x';

  void setRate(double rate) {
    setState(() {
      _rate = rate;
    });
    player.setPlaybackRate(rate);
  }

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final textTheme = Theme.of(context).textTheme;

    return Scaffold(
      backgroundColor: scheme.surface,
      appBar: AppBar(title: const Text('audiopc demo')),
      body: SafeArea(
        child: LayoutBuilder(
          builder: (context, constraints) {
            bool isWide = constraints.maxWidth > 600;
            return Padding(
              padding: const EdgeInsets.all(20.0),
              child: SingleChildScrollView(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text('Audio Source', style: textTheme.titleLarge),
                    const SizedBox(height: 12),
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
                    Row(
                      children: [
                        if (isUrlSource)
                          Expanded(
                            child: TextField(
                              controller: sourceController,
                              decoration: const InputDecoration(
                                border: OutlineInputBorder(),
                                labelText: 'Audio URL',
                              ),
                            ),
                          )
                        else
                          ElevatedButton(
                            onPressed: _selectFile,
                            child: const Text('Select a file'),
                          ),
                        const SizedBox(width: 12),
                        FilledButton(
                          onPressed: loadSource,
                          child: const Text('Load'),
                        ),
                      ],
                    ),
                    const SizedBox(height: 24),
                    if (_metadata != null || _thumbByte != null)
                      Card(
                        elevation: 2,
                        child: Padding(
                          padding: const EdgeInsets.all(16.0),
                          child: isWide
                              ? Row(
                                  crossAxisAlignment: CrossAxisAlignment.start,
                                  children: [
                                    if (_thumbByte != null)
                                      Image.memory(
                                        _thumbByte!,
                                        width: 150,
                                        height: 150,
                                        fit: BoxFit.cover,
                                      ),
                                    if (_thumbByte != null)
                                      const SizedBox(width: 16),
                                    Expanded(
                                      child: _buildMetadataInfo(textTheme),
                                    ),
                                  ],
                                )
                              : Column(
                                  children: [
                                    if (_thumbByte != null)
                                      Image.memory(
                                        _thumbByte!,
                                        width: double.infinity,
                                        height: 200,
                                        fit: BoxFit.cover,
                                      ),
                                    if (_thumbByte != null)
                                      const SizedBox(height: 16),
                                    _buildMetadataInfo(textTheme),
                                  ],
                                ),
                        ),
                      ),
                    const SizedBox(height: 24),
                    _buildPlaybackControls(),
                    const SizedBox(height: 20),
                    Text('Seek position', style: textTheme.titleMedium),
                    _buildSeekSlider(),
                    const SizedBox(height: 20),
                    Text('Rate ${_rateLabel(_rate)}', style: textTheme.titleMedium),
                    Slider(
                      value: _rate,
                      min: 0.5,
                      max: 2.0,
                      divisions: 15,
                      label: _rateLabel(_rate),
                      onChanged: setRate,
                    ),
                    const SizedBox(height: 20),
                    Text(
                      'Volume: ${_volumeLabel(_volumePercent)}',
                      style: textTheme.titleMedium,
                    ),
                    Slider(
                      value: _volumePercent,
                      min: 0,
                      max: 100,
                      divisions: 100,
                      label: _volumeLabel(_volumePercent),
                      onChanged: setVolume,
                    ),
                    const SizedBox(height: 20),
                    Text('Spectrum visualizer', style: textTheme.titleMedium),
                    const SizedBox(height: 8),
                    _buildVisualizer(),
                    const SizedBox(height: 20),
                    _buildBackendInfo(textTheme),
                    const SizedBox(height: 16),
                    FilterControls(player: player),
                  ],
                ),
              ),
            );
          },
        ),
      ),
    );
  }

  Widget _buildMetadataInfo(TextTheme textTheme) {
    if (_metadata == null) {
      return const Text('No metadata loaded');
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Title: ${_metadata!.title}', style: textTheme.titleMedium),
        Text('Artist: ${_metadata!.artist}'),
        Text('Album: ${_metadata!.album}'),
        Text('Genre: ${_metadata!.genre}'),
        Text('Year: ${_metadata!.year}'),
        Text('Track: ${_metadata!.trackNumber}'),
        Text('Disc: ${_metadata!.discNumber}'),
      ],
    );
  }

  Widget _buildPlaybackControls() {
    return Wrap(
      spacing: 12,
      runSpacing: 12,
      alignment: WrapAlignment.center,
      children: [
        StreamBuilder(
          stream: player.stateStream,
          builder: (context, snapshot) {
            final state = snapshot.data ?? PlayerState.stopped;
            return FloatingActionButton(
              onPressed: () {
                switch (state) {
                  case PlayerState.playing:
                    player.pause();
                    break;
                  case PlayerState.paused:
                  case PlayerState.idle:
                    player.play();
                    break;
                  case _:
                    break;
                }
              },
              child: Icon(
                state == PlayerState.playing ? Icons.pause : Icons.play_arrow,
              ),
            );
          },
        ),
        FloatingActionButton(
          onPressed: stop,
          backgroundColor: Colors.red,
          child: const Icon(Icons.stop),
        ),
        OutlinedButton(onPressed: _getThumb, child: const Text('Load Thumb')),
        OutlinedButton(
          onPressed: _getMetadata,
          child: const Text('Get metadata'),
        ),
      ],
    );
  }

  Widget _buildSeekSlider() {
    return StreamBuilder(
      stream: player.positionStream,
      builder: (context, snapshot) {
        final pos = snapshot.data ?? 0;

        return Row(
          children: [
            Text(_formatTime(pos)),
            Expanded(
              child: Slider(
                value: _sliderPosition.clamp(
                  0,
                  player.durationMillis.toDouble().clamp(0, double.maxFinite),
                ),
                min: 0,
                max: player.durationMillis.toDouble().clamp(
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
            Text(_formatTime(player.durationMillis)),
          ],
        );
      },
    );
  }

  Widget _buildVisualizer() {
    return SizedBox(
      height: 150,
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
                  height: 0 + (value * 112).abs(),
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
    );
  }

  Widget _buildBackendInfo(TextTheme textTheme) {
    return ExpansionTile(
      title: Text('Backend Info', style: textTheme.titleMedium),
      children: [
        Text('CPAL backend ready: ${backendInfo.isAvailable}'),
        const SizedBox(height: 8),
        Text('Output sample rate: ${backendInfo.defaultOutputSampleRate}'),
        Text('Output channels: ${backendInfo.defaultOutputChannels}'),
        Text('Output device count: ${backendInfo.outputDeviceCount}'),
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
    );
  }
}
