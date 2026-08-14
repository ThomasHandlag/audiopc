import 'dart:async';
import 'dart:developer';

import 'package:audiopc/audiopc.dart';
import 'package:file_picker/file_picker.dart';
import 'package:material_ui/material_ui.dart' hide MetaData;
import 'shader_widget.dart';

/// Example app entry point.
void main() async {
  await RustLib.init();
  final player = PcPlayer.instance();
  runApp(
    MaterialApp(
      debugShowCheckedModeBanner: false,
      title: 'AudioPC Demo',
      home: MyApp(player: player),
    ),
  );
}

/// Demo app showing source loading, playback control, and visualization.
class MyApp extends StatefulWidget {
  const MyApp({super.key, required this.player});

  final PcPlayer player;

  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  static const int _visualizerFps = 60;
  static const int _spectrumBinCount = 64;
  static const List<String> _eqLabels = [
    '31',
    '62',
    '125',
    '250',
    '500',
    '1k',
    '2k',
    '4k',
    '8k',
    '16k',
  ];

  PcPlayer get player => widget.player;

  final sourceController = TextEditingController();

  late final Timer _visualizerTimer;

  String bufferedSamples = '0';
  String positionMillis = '0';
  String durationMillis = '-1';
  bool isUrlSource = false;
  double _sliderPosition = 0;
  double _volumePercent = 70;
  List<double> _spectrumBars = List<double>.filled(_spectrumBinCount, 0);
  List<double> eqGains = List<double>.filled(10, 0); // -12 to +12 dB
  bool _showShader = false;
  bool _showEqualizer = false;

  bool _showEffects = false;

  // Reverb
  double _reverbRoomSize = 0.5;
  double _reverbDamping = 0.5;
  double _reverbWet = 0.0;

  // Delay
  double _delaySeconds = 0.0;
  double _delayFeedback = 0.0;
  double _delayWet = 0.0;

  // Chorus
  double _chorusRate = 0.0;
  double _chorusDepth = 0.0;
  double _chorusWet = 0.0;

  // Override
  double _overrideDrive = 0.0;
  double _overrideOutput = 0.0;

  late final AudioOutputConfig config;

  @override
  void dispose() {
    _visualizerTimer.cancel();
    sourceController.dispose();
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

    player.setSource(AudioSource.path(source));
    setState(() {});
  }

  /// Starts playback.
  void resume() {
    player.resume();
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
    player.setVolumn(_volumeFromSlider(sliderValue));
  }

  /// Updates spectrum bars from the current visualizer frame.
  void _updateSpectrum() async {
    final next = player.getBars();
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
    if (ms < 0) return '--:--';
    final seconds = ms ~/ 1000;
    final minutes = seconds ~/ 60;
    final secs = seconds % 60;
    return '${minutes.toString().padLeft(2, '0')}:${secs.toString().padLeft(2, '0')}';
  }

  @override
  void initState() {
    super.initState();

    player.setProcesor(128, 4096);

    config = player.getConfig();

    player.set10BandsEqualizer();

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
    log(rate.toString());

    setState(() {
      _rate = rate;
    });
    player.setRate(rate);
  }

  void _setEqGain(int bandIndex, double gain) {
    setState(() {
      eqGains[bandIndex] = gain;
    });

    player.setBand(bandIndex, gain);
  }

  void _applyReverb() {
    player.setReverb(_reverbRoomSize, _reverbDamping, _reverbWet);
  }

  void _applyDelay() {
    player.setDelay(_delaySeconds, _delayFeedback, _delayWet);
  }

  void _applyChorus() {
    player.setChorus(_chorusRate, _chorusDepth, _chorusWet);
  }

  void _applyOverride() {
    player.setOverDride(_overrideDrive, _overrideOutput);
  }

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final textTheme = Theme.of(context).textTheme;

    return Scaffold(
      backgroundColor: scheme.surface,
      appBar: AppBar(
        title: const Text('AUDIOPC'),
        actions: [
          IconButton(
            icon: Icon(_showShader ? Icons.visibility : Icons.visibility_off),
            onPressed: () => setState(() => _showShader = !_showShader),
            tooltip: 'Toggle Shader Visualizer',
          ),
        ],
      ),
      body: SafeArea(
        child: LayoutBuilder(
          builder: (context, constraints) {
            return Padding(
              padding: const EdgeInsets.all(16.0),
              child: SingleChildScrollView(
                child: Column(
                  children: [
                    // ── Audio Source Section ──
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Row(
                          children: [
                            Icon(Icons.library_music, size: 20),
                            const SizedBox(width: 8),
                            Text('Audio Source', style: textTheme.titleLarge),
                          ],
                        ),
                        const SizedBox(height: 12),
                        SegmentedButton<bool>(
                          segments: const [
                            ButtonSegment(
                              value: false,
                              label: Text('Local file'),
                              icon: Icon(Icons.folder_open),
                            ),
                            ButtonSegment(
                              value: true,
                              label: Text('URL stream'),
                              icon: Icon(Icons.link),
                            ),
                          ],
                          selected: {isUrlSource},
                          onSelectionChanged: (selected) {
                            setState(() => isUrlSource = selected.first);
                          },
                        ),
                        const SizedBox(height: 12),
                        Row(
                          children: [
                            Expanded(
                              child: isUrlSource
                                  ? TextField(
                                      controller: sourceController,
                                      decoration: InputDecoration(
                                        border: OutlineInputBorder(
                                          borderRadius: BorderRadius.circular(
                                            12,
                                          ),
                                          borderSide: const BorderSide(
                                            color: Color(0xFF00E5FF),
                                          ),
                                        ),
                                        enabledBorder: OutlineInputBorder(
                                          borderRadius: BorderRadius.circular(
                                            12,
                                          ),
                                          borderSide: BorderSide(
                                            color: const Color(
                                              0xFF00E5FF,
                                            ).withValues(alpha: 0.3),
                                          ),
                                        ),
                                        focusedBorder: OutlineInputBorder(
                                          borderRadius: BorderRadius.circular(
                                            12,
                                          ),
                                          borderSide: const BorderSide(
                                            color: Color(0xFF00E5FF),
                                            width: 2,
                                          ),
                                        ),
                                        labelText: 'Audio URL',
                                        labelStyle: const TextStyle(
                                          color: Color(0xFFB0B0D0),
                                        ),
                                      ),
                                      style: const TextStyle(
                                        color: Color(0xFFE0E0FF),
                                      ),
                                    )
                                  : SizedBox(
                                      width: double.infinity,
                                      child: ElevatedButton.icon(
                                        onPressed: _selectFile,
                                        icon: const Icon(Icons.audiotrack),
                                        label: const Text('Select Audio File'),
                                      ),
                                    ),
                            ),
                            const SizedBox(width: 12),
                            FilledButton.icon(
                              onPressed: loadSource,
                              icon: const Icon(Icons.download, size: 20),
                              label: const Text('Load'),
                            ),
                          ],
                        ),
                      ],
                    ),
                    const SizedBox(height: 16),

                    // ── Playback Controls ──
                    Column(
                      children: [
                        _buildPlaybackControls(),
                        const SizedBox(height: 16),
                        _buildSeekSlider(),
                      ],
                    ),
                    const SizedBox(height: 16),

                    Column(
                      children: [
                        _buildSliderRow(
                          icon: Icons.speed,
                          label: 'Speed',
                          value: _rate,
                          min: 0.5,
                          max: 2.0,
                          divisions: 15,
                          display: _rateLabel(_rate),
                          onChanged: setRate,
                        ),
                        Divider(
                          color: Color(0xFF00E5FF).withValues(alpha: 0.15),
                          height: 24,
                        ),
                        _buildSliderRow(
                          icon: Icons.volume_up,
                          label: 'Volume',
                          value: _volumePercent,
                          min: 0,
                          max: 100,
                          divisions: 100,
                          display: _volumeLabel(_volumePercent),
                          onChanged: setVolume,
                        ),
                      ],
                    ),
                    const SizedBox(height: 16),

                    // ── Shader Visualizer ──
                    if (_showShader)
                      SizedBox(
                        height: 280,
                        child: RotatedBox(
                          quarterTurns: 2,
                          child: ShaderWidget(
                            shaderName: 'bar.frag',
                            spectrum: _spectrumBars,
                            spectrumWidth: _spectrumBars.length,
                            uniforms: [],
                          ),
                        ),
                      )
                    else
                      Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Row(
                            children: [
                              Icon(Icons.bar_chart, size: 20),
                              const SizedBox(width: 8),
                              Text(
                                'Spectrum Visualizer',
                                style: textTheme.titleLarge,
                              ),
                            ],
                          ),
                          const SizedBox(height: 12),
                          _buildSpectrumVisualizer(),
                        ],
                      ),
                    const SizedBox(height: 16),

                    Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        InkWell(
                          onTap: () =>
                              setState(() => _showEqualizer = !_showEqualizer),
                          borderRadius: BorderRadius.circular(8),
                          child: Row(
                            children: [
                              Icon(Icons.tune, size: 20),
                              const SizedBox(width: 8),
                              Text(
                                'Graphic Equalizer',
                                style: textTheme.titleLarge,
                              ),
                              const Spacer(),
                              Icon(
                                _showEqualizer
                                    ? Icons.expand_less
                                    : Icons.expand_more,
                                color: const Color(0xFFB0B0D0),
                              ),
                            ],
                          ),
                        ),
                        if (_showEqualizer) ...[
                          const SizedBox(height: 12),
                          _buildEqualizer(),
                        ],
                      ],
                    ),
                    const SizedBox(height: 16),

                    // ── Effects ──
                    _buildEffectsSection(),
                    const SizedBox(height: 24),
                  ],
                ),
              ),
            );
          },
        ),
      ),
    );
  }

  Widget _buildSliderRow({
    required IconData icon,
    required String label,
    required double value,
    required double min,
    required double max,
    int? divisions,
    required String display,
    required ValueChanged<double> onChanged,
  }) {
    return Row(
      children: [
        Icon(icon, size: 20),
        const SizedBox(width: 8),
        SizedBox(
          width: 52,
          child: Text(label, style: Theme.of(context).textTheme.titleMedium),
        ),
        Expanded(
          child: Slider(
            value: value,
            min: min,
            max: max,
            divisions: divisions,
            label: display,
            onChanged: onChanged,
          ),
        ),
        SizedBox(
          width: 52,
          child: Text(
            display,
            textAlign: TextAlign.right,
            style: const TextStyle(
              fontSize: 13,
              fontWeight: FontWeight.w600,
              fontFeatures: [FontFeature.tabularFigures()],
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildPlaybackControls() {
    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        // Previous track placeholder
        // Play/Pause
        StreamBuilder<PlaybackState>(
          stream: player.stateStream,
          builder: (context, snapshot) {
            final state = snapshot.data ?? PlaybackState.idle;
            final isPlaying = state == PlaybackState.playing;

            return ElevatedButton(
              onPressed: () {
                switch (state) {
                  case PlaybackState.playing:
                    player.pause();
                    break;
                  case PlaybackState.paused:
                    player.resume();
                  case PlaybackState.idle:
                    player.play();
                    break;
                  default:
                    break;
                }
              },
              child: Text(isPlaying ? 'Pause' : 'Play'),
            );
          },
        ),
        const SizedBox(width: 8),

        // Stop
        ElevatedButton(
          onPressed: stop,
          child: const Icon(Icons.stop, color: Color(0xFFFF5252), size: 28),
        ),
        const SizedBox(width: 8),
      ],
    );
  }

  // ─── Seek Slider ─────────────────────────────────────────────────────

  Widget _buildSeekSlider() {
    return StreamBuilder<int>(
      stream: player.positionStream,
      builder: (context, snapshot) {
        final pos = snapshot.data ?? 0;
        final dur = player.duration;

        return Column(
          children: [
            SliderTheme(
              data: SliderTheme.of(context).copyWith(
                trackHeight: 3,
                thumbShape: const RoundSliderThumbShape(enabledThumbRadius: 6),
              ),
              child: Slider(
                value: _sliderPosition.clamp(
                  0,
                  dur.toDouble().clamp(0, double.maxFinite),
                ),
                min: 0,
                max: dur.toDouble().clamp(0, double.maxFinite),
                onChanged: (value) {
                  setState(() => _sliderPosition = value);
                },
                onChangeEnd: seekToMillis,
              ),
            ),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text(
                    _formatTime(pos),
                    style: const TextStyle(
                      color: Color(0xFF8080A0),
                      fontSize: 12,
                      fontFeatures: [FontFeature.tabularFigures()],
                    ),
                  ),
                  Text(
                    _formatTime(dur),
                    style: const TextStyle(
                      color: Color(0xFF8080A0),
                      fontSize: 12,
                      fontFeatures: [FontFeature.tabularFigures()],
                    ),
                  ),
                ],
              ),
            ),
          ],
        );
      },
    );
  }

  // ─── Spectrum Visualizer ─────────────────────────────────────────────

  Widget _buildSpectrumVisualizer() {
    return SizedBox(
      height: 160,
      child: ClipRRect(
        borderRadius: BorderRadius.circular(8),
        child: CustomPaint(
          painter: _SpectrumPainter(bars: _spectrumBars),
          size: Size.infinite,
        ),
      ),
    );
  }

  // ─── Graphic Equalizer ───────────────────────────────────────────────

  Widget _buildEqualizer() {
    return Column(
      children: [
        // Equalizer sliders
        SizedBox(
          height: 200,
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: List.generate(_eqLabels.length, (i) {
              return Expanded(
                child: _buildEqBand(
                  index: i,
                  label: _eqLabels[i],
                  gain: eqGains[i],
                  spectrumValue: i < _spectrumBars.length
                      ? _spectrumBars[(i *
                                _spectrumBars.length /
                                _eqLabels.length)
                            .round()
                            .clamp(0, _spectrumBars.length - 1)]
                      : 0,
                ),
              );
            }),
          ),
        ),
        const SizedBox(height: 8),
        // Frequency labels
        Row(
          children: List.generate(_eqLabels.length, (i) {
            return Expanded(
              child: Text(
                _eqLabels[i],
                textAlign: TextAlign.center,
                style: const TextStyle(
                  color: Color(0xFF8080A0),
                  fontSize: 10,
                  fontWeight: FontWeight.w500,
                ),
              ),
            );
          }),
        ),
        const SizedBox(height: 4),
        Row(
          children: [
            const Spacer(),
            TextButton.icon(
              onPressed: () {
                for (int i = 0; i < eqGains.length; i++) {
                  _setEqGain(i, 0);
                }
              },
              icon: const Icon(Icons.restart_alt, size: 16),
              label: const Text('Reset'),
              style: TextButton.styleFrom(
                foregroundColor: const Color(0xFF8080A0),
              ),
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildEqBand({
    required int index,
    required String label,
    required double gain,
    required double spectrumValue,
  }) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 2),
      child: Column(
        children: [
          // Gain value label
          Text(
            '${gain >= 0 ? '+' : ''}${gain.toStringAsFixed(0)}',
            style: TextStyle(
              color: gain == 0
                  ? const Color(0xFF8080A0)
                  : gain > 0
                  ? const Color(0xFF00E676)
                  : const Color(0xFFFF5252),
              fontSize: 9,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 4),
          // Vertical slider
          Expanded(
            child: RotatedBox(
              quarterTurns: -1,
              child: SliderTheme(
                data: SliderTheme.of(context).copyWith(
                  trackHeight: 3,
                  thumbShape: const RoundSliderThumbShape(
                    enabledThumbRadius: 5,
                  ),
                  activeTrackColor: gain >= 0
                      ? const Color(0xFF00E676)
                      : const Color(0xFFFF5252),

                  thumbColor: gain > 0
                      ? const Color(0xFF00E676)
                      : const Color(0xFFFF5252),
                ),
                child: Slider(
                  value: gain,
                  min: -12,
                  max: 12,
                  divisions: 24,
                  onChanged: (v) => _setEqGain(index, v),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildEffectsSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        InkWell(
          onTap: () => setState(() => _showEffects = !_showEffects),
          borderRadius: BorderRadius.circular(8),
          child: Row(
            children: [
              Icon(Icons.auto_fix_high, size: 20),
              const SizedBox(width: 8),
              Text('Effects', style: Theme.of(context).textTheme.titleLarge),
              const Spacer(),
              Icon(
                _showEffects ? Icons.expand_less : Icons.expand_more,
                color: const Color(0xFFB0B0D0),
              ),
            ],
          ),
        ),
        if (_showEffects) ...[
          const SizedBox(height: 12),
          _buildEffectGroup(
            title: 'Reverb',
            icon: Icons.water_drop,
            children: [
              _buildSliderRow(
                icon: Icons.home,
                label: 'Room',
                value: _reverbRoomSize,
                min: 0,
                max: 1,
                divisions: 20,
                display: _reverbRoomSize.toStringAsFixed(2),
                onChanged: (v) {
                  setState(() => _reverbRoomSize = v);
                  _applyReverb();
                },
              ),
              _buildSliderRow(
                icon: Icons.waves,
                label: 'Damping',
                value: _reverbDamping,
                min: 0,
                max: 1,
                divisions: 20,
                display: _reverbDamping.toStringAsFixed(2),
                onChanged: (v) {
                  setState(() => _reverbDamping = v);
                  _applyReverb();
                },
              ),
              _buildSliderRow(
                icon: Icons.water_drop,
                label: 'Wet',
                value: _reverbWet,
                min: 0,
                max: 1,
                divisions: 20,
                display: _reverbWet.toStringAsFixed(2),
                onChanged: (v) {
                  setState(() => _reverbWet = v);
                  _applyReverb();
                },
              ),
            ],
          ),
          Divider(height: 24),
          _buildEffectGroup(
            title: 'Delay',
            icon: Icons.timer,
            children: [
              _buildSliderRow(
                icon: Icons.access_time,
                label: 'Time',
                value: _delaySeconds,
                min: 0,
                max: 1,
                divisions: 20,
                display: '${(_delaySeconds * 1000).toInt()}ms',
                onChanged: (v) {
                  setState(() => _delaySeconds = v);
                  _applyDelay();
                },
              ),
              _buildSliderRow(
                icon: Icons.replay,
                label: 'Feedback',
                value: _delayFeedback,
                min: 0,
                max: 1,
                divisions: 20,
                display: _delayFeedback.toStringAsFixed(2),
                onChanged: (v) {
                  setState(() => _delayFeedback = v);
                  _applyDelay();
                },
              ),
              _buildSliderRow(
                icon: Icons.water_drop,
                label: 'Wet',
                value: _delayWet,
                min: 0,
                max: 1,
                divisions: 20,
                display: _delayWet.toStringAsFixed(2),
                onChanged: (v) {
                  setState(() => _delayWet = v);
                  _applyDelay();
                },
              ),
            ],
          ),
          Divider(height: 24),
          _buildEffectGroup(
            title: 'Chorus',
            icon: Icons.audiotrack,
            children: [
              _buildSliderRow(
                icon: Icons.speed,
                label: 'Rate',
                value: _chorusRate,
                min: 0,
                max: 10,
                divisions: 20,
                display: '${_chorusRate.toStringAsFixed(1)}Hz',
                onChanged: (v) {
                  setState(() => _chorusRate = v);
                  _applyChorus();
                },
              ),
              _buildSliderRow(
                icon: Icons.arrow_downward,
                label: 'Depth',
                value: _chorusDepth,
                min: 0,
                max: 1,
                divisions: 20,
                display: _chorusDepth.toStringAsFixed(2),
                onChanged: (v) {
                  setState(() => _chorusDepth = v);
                  _applyChorus();
                },
              ),
              _buildSliderRow(
                icon: Icons.water_drop,
                label: 'Wet',
                value: _chorusWet,
                min: 0,
                max: 1,
                divisions: 20,
                display: _chorusWet.toStringAsFixed(2),
                onChanged: (v) {
                  setState(() => _chorusWet = v);
                  _applyChorus();
                },
              ),
            ],
          ),
          Divider(height: 24),
          _buildEffectGroup(
            title: 'Override',
            icon: Icons.tune,
            children: [
              _buildSliderRow(
                icon: Icons.grain,
                label: 'Drive',
                value: _overrideDrive,
                min: 0,
                max: 1,
                divisions: 20,
                display: _overrideDrive.toStringAsFixed(2),
                onChanged: (v) {
                  setState(() => _overrideDrive = v);
                  _applyOverride();
                },
              ),
              _buildSliderRow(
                icon: Icons.volume_up,
                label: 'Output',
                value: _overrideOutput,
                min: 0,
                max: 1,
                divisions: 20,
                display: _overrideOutput.toStringAsFixed(2),
                onChanged: (v) {
                  setState(() => _overrideOutput = v);
                  _applyOverride();
                },
              ),
            ],
          ),
        ],
      ],
    );
  }

  Widget _buildEffectGroup({
    required String title,
    required IconData icon,
    required List<Widget> children,
  }) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Icon(icon, size: 18),
            const SizedBox(width: 8),
            Text(title, style: Theme.of(context).textTheme.titleMedium),
          ],
        ),
        const SizedBox(height: 8),
        ...children,
      ],
    );
  }
}

// ─── Custom Spectrum Painter ───────────────────────────────────────────

class _SpectrumPainter extends CustomPainter {
  final List<double> bars;

  _SpectrumPainter({required this.bars});

  @override
  void paint(Canvas canvas, Size size) {
    final barCount = bars.length;
    if (barCount == 0) return;

    final barWidth = size.width / barCount;
    final gap = barWidth * 0.15;
    final paint = Paint();

    for (int i = 0; i < barCount; i++) {
      final value = bars[i].clamp(0.0, 1.0);
      final barHeight = value * size.height;

      final x = i * barWidth + gap / 2;
      final y = size.height - barHeight;

      // Gradient from cyan to green based on value
      final color = Color.lerp(Colors.white, Colors.black, value)!;

      paint.color = color.withValues(alpha: 0.7 + value * 0.3);
      paint.style = PaintingStyle.fill;

      canvas.drawRRect(
        RRect.fromRectAndRadius(
          Rect.fromLTWH(x, y, barWidth - gap, barHeight),
          const Radius.circular(2),
        ),
        paint,
      );

      // Glow line at top of bar
      if (barHeight > 2) {
        paint.color = Colors.white.withValues(alpha: 0.3);
        canvas.drawRRect(
          RRect.fromRectAndRadius(
            Rect.fromLTWH(x, y, barWidth - gap, 2),
            const Radius.circular(1),
          ),
          paint,
        );
      }
    }

    // Grid lines
    final gridPaint = Paint()
      ..color = const Color(0xFF00E5FF).withValues(alpha: 0.05)
      ..strokeWidth = 1;
    for (int i = 0; i < 4; i++) {
      final y = size.height * (i + 1) / 5;
      canvas.drawLine(Offset(0, y), Offset(size.width, y), gridPaint);
    }
  }

  @override
  bool shouldRepaint(_SpectrumPainter oldDelegate) {
    return oldDelegate.bars != bars;
  }
}
