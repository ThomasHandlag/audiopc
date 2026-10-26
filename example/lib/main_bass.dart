import 'dart:ui';

import 'package:material_ui/material_ui.dart';
import 'dart:async';
import 'dart:math';

import 'package:audiopc/audiopc.dart';

void main() async {
  await RustLib.init();
  runApp(MaterialApp(home: VisualizerDemo()));
}

// ──────────────────────────────────────────────────────────────────────
// visualizer_demo.dart
// ──────────────────────────────────────────────────────────────────────

/// A complete demo that
///   • starts a player,
///   • extracts drum / bass / hi‑freq / lo‑freq / vocal metrics,
///   • draws a pulsating circle (drum), a sine wave (bass) and three bars.
class VisualizerDemo extends StatefulWidget {
  const VisualizerDemo({super.key});

  @override
  State<VisualizerDemo> createState() => _VisualizerDemoState();
}

class _VisualizerDemoState extends State<VisualizerDemo>
    with SingleTickerProviderStateMixin {
  // -----------------------------------------------------------------
  // 1️⃣ Player & polling
  // -----------------------------------------------------------------
  late final PcPlayer _player;
  late Timer _pollTimer;

  // Metrics returned by the visualiser (range 0‑1)
  double _drum = 0.0;
  double _bass = 0.0;
  double _hi = 0.0;
  double _lo = 0.0;
  double _vocal = 0.0;

  // Used only by the custom painter to animate smoothly.
  // It stores the *last* value so that the circle / sine does not jump
  // abruptly when the timer fires.
  double _drumAnim = 0.0;
  double _bassAnim = 0.0;

  // -----------------------------------------------------------------
  // 2️⃣ Lifecycle
  // -----------------------------------------------------------------
  @override
  void initState() {
    super.initState();
    _initPlayer();
  }

  Future<void> _initPlayer() async {
    // Grab the singleton player.
    _player = PcPlayer.instance();

    // Choose a resolution (64 bars, 1024‑point FFT). One‑time init.
    _player.setProcesor(64, 1024);

    // Load & start a track. Replace the path with a real asset or URL.
    _player.playSource(AudioSource.path('~/Downloads/Post spring.mp3'));

    // Poll visualiser data ~30 Hz.
    _pollTimer = Timer.periodic(const Duration(milliseconds: 33), (_) {
      _updateMetrics();
    });
  }

  // -----------------------------------------------------------------
  // 3️⃣ Metric extraction
  // -----------------------------------------------------------------
  void _updateMetrics() {
    // Select the bands that are useful for the drum‑beat calculation.
    final wanted = <VBandKind>[
      .toms
    ];

    _player.getBars();
    final List<BandEnergy> bands = _player.getBands(wanted);

    // High‑level helpers (thin wrappers around the Rust static methods).
    final double drum = VisualizerProcessor.drumBeat(bands: bands);
    final double bass = VisualizerProcessor.bassBeat(bands: bands);
    final double hi = VisualizerProcessor.highFrequencyEnergy(bands: bands);
    final double lo = VisualizerProcessor.lowFrequencyEnergy(bands: bands);
    final double vocal = VisualizerProcessor.vocalPresence(bands: bands);

    // Update UI state *and* the animated copies.
    setState(() {
      _drum = drum;
      _bass = bass;
      _hi = hi;
      _lo = lo;
      _vocal = vocal;

      // Smooth the animation a little (simple exponential smoothing).
      const double smoothing = 0.2;
      _drumAnim = _drumAnim * (1 - smoothing) + drum * smoothing;
      _bassAnim = _bassAnim * (1 - smoothing) + bass * smoothing;
    });
  }

  // -----------------------------------------------------------------
  // 4️⃣ Cleanup
  // -----------------------------------------------------------------
  @override
  void dispose() {
    _pollTimer.cancel();
    _player.dispose();
    super.dispose();
  }

  // -----------------------------------------------------------------
  // 5️⃣ UI
  // -----------------------------------------------------------------
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF111111),
      body: SafeArea(
        child: Column(
          children: [
            // ── Visual area (circle + sine) ──
            Expanded(
              child: CustomPaint(
                painter: _VisualizerPainter(drum: _drumAnim, bass: _bassAnim),
                child: const SizedBox.expand(),
              ),
            ),

            // ── Metric bars (hi / lo / vocal) ──
            const Padding(
              padding: EdgeInsets.symmetric(vertical: 8.0),
              child: Text(
                'Visualizer metrics (0‑1)',
                style: TextStyle(color: Colors.white70, fontSize: 16),
              ),
            ),
            _meter('Hi‑freq', _hi, Colors.lightBlueAccent),
            _meter('Low‑freq', _lo, Colors.greenAccent),
            _meter('Vocal', _vocal, Colors.purpleAccent),
            const SizedBox(height: 16),
          ],
        ),
      ),
    );
  }

  // -----------------------------------------------------------------
  // 6️⃣ Helper: progress‑bar row
  // -----------------------------------------------------------------
  Widget _meter(String label, double value, Color color) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4, horizontal: 12),
      child: Row(
        children: [
          SizedBox(
            width: 70,
            child: Text(label, style: const TextStyle(color: Colors.white70)),
          ),
          Expanded(
            child: LinearProgressIndicator(
              value: value,
              minHeight: 10,
              backgroundColor: Colors.black12,
              valueColor: AlwaysStoppedAnimation(color),
            ),
          ),
          const SizedBox(width: 8),
          Text(
            value.toStringAsFixed(2),
            style: const TextStyle(color: Colors.white70),
          ),
          IconButton(
            onPressed: () {
              _player.playSource(
                AudioSource.path('/home/thuong/Downloads/Post spring.mp3'),
              );
            },
            icon: Icon(Icons.pause),
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------
// Custom painter that draws:
//   • a pulsating circle (drum)
//   • a sine wave whose amplitude follows the bass metric
// ---------------------------------------------------------------------
class _VisualizerPainter extends CustomPainter {
  final double drum; // 0‑1, smooth animated value
  final double bass; // 0‑1, smooth animated value

  _VisualizerPainter({required this.drum, required this.bass});

  @override
  void paint(Canvas canvas, Size size) {
    // Centre of the canvas
    final Offset centre = size.center(Offset.zero);
    final Paint paint = Paint()..style = PaintingStyle.stroke;

    // -------------------------------------------------------------
    // 1️⃣ Drum circle – radius = base + drum * scale
    // -------------------------------------------------------------
    const double baseRadius = 40.0;
    const double drumScale = 80.0;
    final double radius = baseRadius + drum * drumScale;

    paint
      ..color = Colors.redAccent.withOpacity(0.7)
      ..strokeWidth = 6;
    canvas.drawCircle(centre, radius, paint);

    // -------------------------------------------------------------
    // 2️⃣ Bass sine wave – draws across the full width
    // -------------------------------------------------------------
    final Path sinePath = Path();
    const int points = 100;
    final double waveHeight = 30.0; // max vertical displacement
    final double amplitude = bass * waveHeight; // 0‑30px

    for (int i = 0; i <= points; i++) {
      final double t = i / points; // 0‑1
      // Horizontal position
      final double x = lerpDouble(0, size.width, t)!;
      // Simple sinusoid (one full period)
      final double y =
          centre.dy +
          amplitude *
              sin(2 * pi * t + _timeFactor()); // adds a tiny phase shift
      if (i == 0) {
        sinePath.moveTo(x, y);
      } else {
        sinePath.lineTo(x, y);
      }
    }

    paint
      ..color = Colors.orangeAccent.withOpacity(0.9)
      ..strokeWidth = 4;
    canvas.drawPath(sinePath, paint);
  }

  // A very cheap time factor so the sine wave slowly drifts.
  double _timeFactor() => DateTime.now().millisecond / 1000.0;

  @override
  bool shouldRepaint(covariant _VisualizerPainter old) {
    // Repaint whenever the animated values change.
    return old.drum != drum || old.bass != bass;
  }
}
