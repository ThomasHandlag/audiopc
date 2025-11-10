import 'dart:math';
import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:fftea/fftea.dart';

class VisualzerPainter extends CustomPainter with SpectrumProcessor {
  VisualzerPainter({
    required this.clipper,
    required this.deltaTime,
    required this.data,
    required this.isPlaying,
  });
  final CustomClipper<Path> clipper;
  final List<double> data;
  final double deltaTime;
  final bool isPlaying;

  @override
  void paint(Canvas canvas, Size size) {
    var path = clipper.getClip(size);
    const barCount = 64;
    
    final Paint backgroundPaint = Paint()
      ..color = Colors.black.withAlpha(127)
      ..style = PaintingStyle.fill;
    canvas.drawPath(path, backgroundPaint);

    if (data.isEmpty) {
      _drawShadow(canvas, path);
      return;
    }

    // Pre-calculate dimensions
    final barWidth = size.width / (barCount * 1.5);
    final spacing = barWidth / 2;
    final barStep = barWidth + spacing;
    final barWidthHalf = barWidth / 2;

    final barPainter = Paint()
      ..color = const Color.fromARGB(220, 87, 255, 36)
      ..strokeWidth = barWidth
      ..shader = const LinearGradient(
        colors: [
          Color.fromARGB(255, 31, 236, 255),
          Color.fromARGB(255, 87, 255, 36),
        ],
        begin: Alignment.topLeft,
        end: Alignment.bottomRight,
      ).createShader(Rect.fromLTWH(0, 0, size.width, size.height))
      ..strokeJoin = StrokeJoin.round
      ..style = PaintingStyle.fill;

    final maxPeaks = getPeaks(data, barCount);
    
    // Find max once instead of in loop
    final maxPeak = maxPeaks.reduce(max);
    if (maxPeak == 0) {
      _drawShadow(canvas, path);
      return;
    }
    
    final heightScale = size.height / maxPeak;
    final timeMultiplier = isPlaying ? deltaTime : 1.0;

    for (int i = 0; i < barCount; i++) {
      var barHeight = maxPeaks[i] * heightScale * timeMultiplier;

      // Clamp in one step
      barHeight = barHeight.clamp(0.0, size.height);
      
      final x = i * barStep + barWidthHalf;

      canvas.drawLine(
        Offset(x, size.height),
        Offset(x, size.height - barHeight),
        barPainter,
      );
    }

    _drawShadow(canvas, path);
  }
  
  void _drawShadow(Canvas canvas, Path path) {
    final Paint shadowPaint = Paint()
      ..color = Colors.black.withAlpha(127)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 5)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 5;

    path.shift(const Offset(0, 10));
    canvas.drawPath(path, shadowPaint);
  }

  @override
  bool shouldRepaint(VisualzerPainter oldDelegate) =>
      isPlaying && data != oldDelegate.data;
}

class VisualizerClipper extends CustomClipper<Path> {
  final double? radius;
  const VisualizerClipper({this.radius = 10.0});
  @override
  Path getClip(Size size) {
    // final double xScaling = size.width / 414;
    // final double yScaling = size.height / 896;

    // Create a rectangle with rounded corners using RRect
    final rrect = RRect.fromRectAndRadius(
      Rect.fromLTWH(0, 0, size.width, size.height),
      Radius.circular(radius!),
    );

    final path = Path()..addRRect(rrect);
    path.close();
    return path;
  }

  @override
  bool shouldReclip(covariant CustomClipper<Path> oldClipper) => false;
}

class CircleAudioVisualizerPainter extends CustomPainter
    with SpectrumProcessor {
  List<double> data;
  double dy;
  bool isPlaying;
  int currentPosition;
  int numbars;
  final Color? color;

  CircleAudioVisualizerPainter(
    this.dy,
    this.data,
    this.isPlaying,
    this.currentPosition,
    this.numbars, {
    this.color,
  });

  @override
  void paint(Canvas canvas, Size size) {
    const barCount = 64;
    const barWidth = 6.0;
    const baseRadius = 90.0;
    const maxBarHeight = 100.0;
    
    final double centerX = size.width / 2;
    final double centerY = size.height / 2;

    if (data.isEmpty) {
      // Draw static circles when no data
      _drawStaticCircles(canvas, centerX, centerY);
      return;
    }

    final maxPeaks = getPeaks(data, barCount);
    
    // Find max peak once instead of calling reduce in loop
    final maxPeak = maxPeaks.reduce(max);
    if (maxPeak == 0) {
      _drawStaticCircles(canvas, centerX, centerY);
      return;
    }
    
    // Pre-calculate scale factor
    final scaleFactor = size.height / maxPeak;
    final dyMultiplier = isPlaying ? dy : 0.0;
    
    var radius = baseRadius + (maxPeaks[20] * dyMultiplier);
    radius = radius.clamp(0.0, maxBarHeight);

    final paint = Paint()
      ..strokeCap = StrokeCap.round
      ..style = PaintingStyle.stroke
      ..color = color?.withAlpha(255) ?? const Color.fromARGB(255, 84, 33, 61)
      ..strokeWidth = barWidth;

    // Pre-calculate angle increment
    const angleIncrement = 2 * pi / barCount;
    
    for (int i = 0; i < barCount; i++) {
      var barHeight = maxPeaks[i] * scaleFactor * dyMultiplier;
      
      // Clamp in one step
      barHeight = barHeight.clamp(0.0, maxBarHeight);

      final double angle = angleIncrement * i;
      final cosAngle = cos(angle);
      final sinAngle = sin(angle);
      
      final double startX = centerX + baseRadius * cosAngle;
      final double startY = centerY + baseRadius * sinAngle;
      final double endX = centerX + (baseRadius + barHeight) * cosAngle;
      final double endY = centerY + (baseRadius + barHeight) * sinAngle;

      canvas.drawLine(
        Offset(startX, startY),
        Offset(endX, endY),
        paint,
      );
    }

    canvas.drawCircle(Offset(centerX, centerY), radius, paint..strokeWidth = 1);

    final paint2 = Paint()
      ..style = PaintingStyle.stroke
      ..color = Colors.white.withAlpha(100)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 10)
      ..strokeWidth = 6;

    canvas.drawCircle(Offset(centerX, centerY), baseRadius, paint2);
  }
  
  void _drawStaticCircles(Canvas canvas, double centerX, double centerY) {
    const baseRadius = 90.0;
    
    final paint = Paint()
      ..style = PaintingStyle.stroke
      ..color = color?.withAlpha(255) ?? const Color.fromARGB(255, 84, 33, 61)
      ..strokeWidth = 1;
    
    canvas.drawCircle(Offset(centerX, centerY), baseRadius, paint);
    
    final paint2 = Paint()
      ..style = PaintingStyle.stroke
      ..color = Colors.white.withAlpha(100)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 10)
      ..strokeWidth = 6;

    canvas.drawCircle(Offset(centerX, centerY), baseRadius, paint2);
  }

  @override
  bool shouldRepaint(CircleAudioVisualizerPainter oldDelegate) =>
      isPlaying && data != oldDelegate.data;
}

mixin class SpectrumProcessor {
  // Cache STFT and window instances to avoid recreating them on every call
  STFT? _cachedStft;
  List<double>? _cachedFreqBin;
  static const int _stftSize = 4096;
  static const int _barCount = 65;
  static const double _fitFactor = 0.9;
  
  STFT get _stft {
    _cachedStft ??= STFT(_stftSize, Window.hanning(_stftSize));
    return _cachedStft!;
  }
  
  List<double> get _freqBin {
    if (_cachedFreqBin == null) {
      _cachedFreqBin = List.generate(_barCount, (i) {
        double t = i / _barCount;
        return 15.0 * pow(25000.0 / 15.0, t);
      });
    }
    return _cachedFreqBin!;
  }

  List<double> getPeaks(List<double> data, int barCount) {
    if (data.isEmpty) {
      return List.filled(64, 0.0);
    }
    
    final spectrogram = <Float64List>[];
    final freqBin = _freqBin;

    // Run STFT
    _stft.run(data, (Float64x2List freq) {
      spectrogram.add(freq.discardConjugates().squareMagnitudes());
    });

    final lastSpectrum = List.filled(64, 0.0);
    final frequencyScale = data.length / _stftSize.toDouble();
    
    for (var frame in spectrogram) {
      final frameLength = frame.length;
      for (int j = 0; j < frameLength; j++) {
        final magnitude = frame[j];
        final freq = j * frequencyScale;

        // Binary search would be better for large ranges, but for 64 bins linear is fine
        for (int i = 0; i < 64; i++) {
          if (freq >= freqBin[i] && freq <= freqBin[i + 1]) {
            final smoothedValue = max(magnitude, lastSpectrum[i] * _fitFactor);
            lastSpectrum[i] = smoothedValue.abs();
            break; // Early exit once we find the bin
          }
        }
      }
    }
    return lastSpectrum;
  }
  
  void dispose() {
    _cachedStft = null;
    _cachedFreqBin = null;
  }
}

enum SoundRange { bass, subBass, mid, lowMid, upMid, treble, ultra }

class Visualizer extends StatefulWidget {
  const Visualizer({
    super.key,
    required this.child,
    required this.isPlaying,
    this.type = VisualizerType.bar,
    this.color,
    required this.dataStream,
  });
  final Widget child;
  final bool isPlaying;
  final VisualizerType? type;
  final Stream<List<double>> dataStream;
  final Color? color;
  @override
  State<Visualizer> createState() => _VisualizerState();
}

class _VisualizerState extends State<Visualizer>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _animation;
  late Stream<List<double>> _broadcastStream;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 100),
      animationBehavior: AnimationBehavior.preserve,
    );
    _controller.repeat(reverse: true);
    _animation = Tween<double>(begin: 0, end: 1).animate(_controller);
    // Cache the broadcast stream to avoid recreating it on every build
    _broadcastStream = widget.dataStream.asBroadcastStream();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return StreamBuilder(
      stream: _broadcastStream,
      builder: (_, snapshot) {
        return AnimatedBuilder(
          animation: _animation,
          builder: (_, __) {
            return CustomPaint(
              painter: CircleAudioVisualizerPainter(
                _animation.value,
                color: widget.color,
                snapshot.data ?? [],
                widget.isPlaying,
                0,
                64,
              ),
              child: widget.child,
            );
          },
        );
      },
    );
  }
}

enum VisualizerType { circle, bar, wave }
