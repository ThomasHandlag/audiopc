import 'dart:math';
import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:fftea/fftea.dart';

class VisualzerPainter extends CustomPainter with SpectrumProcessor {
  VisualzerPainter(
      {required this.clipper,
      required this.deltaTime,
      required this.data,
      required this.isPlaying});
  final CustomClipper<Path> clipper;
  final List<double> data;
  final double deltaTime;
  final bool isPlaying;

  @override
  void paint(Canvas canvas, Size size) {
    var path = clipper.getClip(size);
    int barCount = 64;
    final Paint backgroundPaint = Paint()
      ..color = Colors.black45
      ..style = PaintingStyle.fill;
    canvas.drawPath(path, backgroundPaint);

    final barPainter = Paint()
      ..color = const Color.fromARGB(220, 50, 234, 255)
      ..style = PaintingStyle.fill;

    final maxPeaks = getPeaks(data, barCount);

    double barWidth = size.width / (barCount * 1.5);
    double spacing = barWidth / 2; // Spacing between bars

    for (int i = 0; i < barCount; i++) {
      final value = maxPeaks[i] * size.height / maxPeaks.reduce(max) * 30;
      double barHeight = value * deltaTime / 30;
      if (barHeight > size.height) {
        barHeight = size.height;
      }
      double x = i * (barWidth + spacing);
      final Rect barRect =
          Rect.fromLTWH(x, size.height - barHeight, barWidth, barHeight);
      canvas.drawRect(barRect, barPainter);
    }

    final Paint shadowPaint = Paint()
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

class CircleAudioVisualizerPainter extends CustomPainter with SpectrumProcessor {
  List<double> data;

  final colors = <Color>[
    Colors.cyan.shade500,
    Colors.blue.shade500,
    Colors.purple.shade500,
    Colors.pink.shade300,
    Colors.red.shade500,
    Colors.orange.shade500,
    Colors.yellow.shade500,
    Colors.green.shade500,
    Colors.brown.shade500,
  ];

  double dy;
  bool isPlaying;
  int currentPosition;
  int numbars;

  CircleAudioVisualizerPainter(
      this.dy, this.data, this.isPlaying, this.currentPosition, this.numbars);

  @override
  void paint(Canvas canvas, Size size) {
    var paint = Paint();
    paint..strokeCap = StrokeCap.round
    ..style = PaintingStyle.stroke;

    const barCount = 64;
    final barWidth = 6.0;
    final double centerX = size.width / 2;
    final double centerY = size.height / 2;

    final maxPeaks = getPeaks(data, barCount);
    double radius = 90 + maxPeaks[49] * dy;
    
    // paint.color = const Color(0xFF1001FF);
    paint.color = colors[maxPeaks[50].toInt() % colors.length];

    for (int i = 0; i < barCount; i++) {
      var barHeight = maxPeaks[i] * dy;
      if  (barHeight > 100) {
        barHeight = 100;
      }
      final double angle = (2 * pi / barCount) * i;
      final double startX = centerX + radius * cos(angle);
      final double startY = centerY + radius * sin(angle);
      final double endX = centerX + (radius + barHeight) * cos(angle);
      final double endY = centerY + (radius + barHeight) * sin(angle);

      canvas.drawLine(
        Offset(startX, startY),
        Offset(endX, endY),
        paint..strokeWidth = barWidth,
      );
    }
    const dradius = 120.0;
    final Path path = Path();
    for (int i = 0; i < 64; i++) {
      final double startAngle = i * (pi / 2) + dy * maxPeaks[i];
      final double endAngle = (i + 1) * (pi / 2) + dy * maxPeaks[i];

      final Offset startPoint = Offset(
        centerX + dradius * cos(startAngle),
        centerY + dradius * sin(startAngle),
      );
      final Offset endPoint = Offset(
        centerX + dradius * cos(endAngle),
        centerY + dradius * sin(endAngle),
      );

      final Offset controlPoint1 = Offset(
        centerX + dradius * cos(startAngle + pi / 6) / cos(pi / 6),
        centerY + dradius * sin(startAngle + pi / 6) / cos(pi / 6),
      );
      final Offset controlPoint2 = Offset(
        centerX + dradius * cos(endAngle - pi / 6) / cos(pi / 6),
        centerY + dradius * sin(endAngle - pi / 6) / cos(pi / 6),
      );

      if (i == 0) {
        path.moveTo(startPoint.dx, startPoint.dy);
      }

      path.cubicTo(
        controlPoint1.dx,
        controlPoint1.dy,
        controlPoint2.dx,
        controlPoint2.dy,
        endPoint.dx,
        endPoint.dy,
      );
    }
    final dpaint = Paint()
      ..color = colors[maxPeaks[50].toInt() % colors.length]
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4)
      ..style = PaintingStyle.fill;
    canvas.drawPath(path, dpaint);
    paint.style = PaintingStyle.fill;
    paint.color = const Color.fromARGB(255, 0, 0, 0);
    canvas.drawCircle(Offset(centerX, centerY), radius, paint);

    final paint2 = Paint()
      ..style = PaintingStyle.stroke
      ..color = const Color(0xF0FFFFFF)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 10)
      ..strokeWidth = 6;

    canvas.drawCircle(Offset(centerX, centerY), radius, paint2);
  }

  @override
  bool shouldRepaint(CircleAudioVisualizerPainter oldDelegate) => true;
}

mixin class SpectrumProcessor {
  List<double> getPeaks(List<double> data, int barCount) {
    final stft = STFT(2048, Window.hanning(2048));
    final spectrogram = <Float64List>[];
    // final freqBin = List.generate(128, (i) => 44100 / 2 * i / 128);

    final freqBin = List.generate(128, (i) {
      double t = i / 128;
      return 30.0 * pow(30000.0 / 30.0, t);
    });

    // Run STFT
    stft.run(data, (Float64x2List freq) {
      spectrogram.add(freq.discardConjugates().magnitudes());
    });

    double fitFactor = 0.3;
    List<double> lastSpectrum = List.filled(128, 0.0);
    for (var frame in spectrogram) {
      for (int j = 0; j < (frame.length ~/ 2); j++) {
        double magnitude = frame[j];
        double freq = j * 44800 / 2048;

        for (int i = 0; i < 128; i++) {
          if (freq >= freqBin[i] && freq <= freqBin[i + 1]) {
            double smoothedValue = max(magnitude, lastSpectrum[i] * fitFactor);
            lastSpectrum[i] = smoothedValue;
          }
        }
      }
    }
    return lastSpectrum;
  }
}
