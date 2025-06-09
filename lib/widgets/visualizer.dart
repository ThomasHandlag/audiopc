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
    int barCount = 64;
    final Paint backgroundPaint =
        Paint()
          ..color = Colors.black.withAlpha(255 ~/ 2)
          ..style = PaintingStyle.fill;
    canvas.drawPath(path, backgroundPaint);

    // Spacing between bars
    double barWidth = size.width / (barCount * 1.5);
    double spacing = barWidth / 2;

    final barPainter =
        Paint()
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

    for (int i = 0; data.isNotEmpty & (i < barCount); i++) {
      final value = maxPeaks[i] * size.height / maxPeaks.reduce(max);
      double barHeight = value * (isPlaying ? deltaTime : 1);

      if ((barHeight.isNaN)) {
        barHeight = 0;
      }

      if (barHeight > size.height) {
        barHeight = size.height;
      }
      double x = i * (barWidth + spacing);

      final startOffset = Offset(x + barWidth / 2, size.height);
      final endOffset = Offset(x + barWidth / 2, size.height - barHeight);

      canvas.drawLine(startOffset, endOffset, barPainter);
    }

    final Paint shadowPaint =
        Paint()
          ..color = Colors.black.withAlpha(255 ~/ 2)
          ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 5)
          ..style = PaintingStyle.stroke
          ..strokeWidth = 5;

    path.shift(const Offset(-0, 10));
    // Draw the shadow
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
    var paint = Paint();
    paint
      ..strokeCap = StrokeCap.round
      ..style = PaintingStyle.stroke;

    const barCount = 64;
    final barWidth = 6.0;
    final double centerX = size.width / 2;
    final double centerY = size.height / 2;

    final maxPeaks = getPeaks(data, barCount);
    double radius =
        90 + ((data.isNotEmpty ? maxPeaks[20] : 0) * (isPlaying ? dy : 0));
    // paint.color = const Color(0xFF1001FF);
    paint.color = color?.withAlpha(255) ?? Color.fromARGB(255, 84, 33, 61);

    for (int i = 0; data.isNotEmpty & (i < barCount); i++) {
      final value = maxPeaks[i] * size.height / maxPeaks.reduce(max);
      var barHeight = value * (isPlaying ? dy : 0);

      if ((barHeight.isNaN)) {
        barHeight = 0;
      }

      if ((barHeight > 100)) {
        barHeight = 100;
      }

      final double angle = (2 * pi / barCount) * i;
      final double startX = centerX + 90 * cos(angle);
      final double startY = centerY + 90 * sin(angle);
      final double endX = centerX + (90 + barHeight) * cos(angle);
      final double endY = centerY + (90 + barHeight) * sin(angle);

      canvas.drawLine(
        Offset(startX, startY),
        Offset(endX, endY),
        paint..strokeWidth = barWidth,
      );
    }

    if (radius > 100 || radius.isNaN) {
      radius = 100;
    }

    canvas.drawCircle(Offset(centerX, centerY), radius, paint..strokeWidth = 1);

    final paint2 =
        Paint()
          ..style = PaintingStyle.stroke
          ..color = Colors.white.withAlpha(100)
          ..maskFilter = MaskFilter.blur(BlurStyle.normal, 10)
          ..strokeWidth = 6;

    canvas.drawCircle(Offset(centerX, centerY), 90, paint2);
  }

  @override
  bool shouldRepaint(CircleAudioVisualizerPainter oldDelegate) =>
      isPlaying && data != oldDelegate.data;
}

mixin class SpectrumProcessor {
  List<double> getPeaks(List<double> data, int barCount) {
    final stft = STFT(4096, Window.hanning(4096));
    final spectrogram = <Float64List>[];

    final freqBin = List.generate(65, (i) {
      double t = i / 65;
      return 15.0 * pow(25000.0 / 15.0, t);
    });

    // Run STFT
    stft.run(data, (Float64x2List freq) {
      spectrogram.add(freq.discardConjugates().squareMagnitudes());
    });

    double fitFactor = 0.9;
    List<double> lastSpectrum = List.filled(65, 0.0);
    for (var frame in spectrogram) {
      for (int j = 0; j < (frame.length); j++) {
        double magnitude = frame[j];
        double freq = j * data.length / 4096;

        for (int i = 0; i < 64; i++) {
          if (freq >= freqBin[i] && freq <= freqBin[i + 1]) {
            double smoothedValue = max(magnitude, lastSpectrum[i] * fitFactor);
            lastSpectrum[i] = smoothedValue.abs();
          }
        }
      }
    }
    return lastSpectrum;
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
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return StreamBuilder(
      stream: widget.dataStream.asBroadcastStream(),
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
