import 'dart:math';
import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:fftea/fftea.dart';

class VisualzerPainter extends CustomPainter {
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

    const chunkSize = 1024;
    final stft = STFT(chunkSize, Window.hanning(chunkSize));
    final spectrogram = <Float64List>[];
    stft.run(data, (Float64x2List freq) {
      spectrogram.add(freq.discardConjugates().magnitudes());
    });

    final maxPeaks = List<double>.filled(barCount, 0.0);
    final binSize = spectrogram[0].length ~/ barCount;

    for (final frame in spectrogram) {
      for (int i = 0; i < barCount; i++) {
        final startIdx = i * binSize;
        final endIdx = (i + 1) * binSize;
        final bin = frame.sublist(startIdx, endIdx);

        final peak =
            bin.reduce((value, element) => element > value ? element : value);
        maxPeaks[i] = maxPeaks[i] > peak ? maxPeaks[i] : peak;
      }
    }
    double barWidth = size.width / (barCount * 1.5);
    double spacing = barWidth / 2; // Spacing between bars

    for (int i = 0; i < barCount; i++) {
      final value = maxPeaks[i] * size.height / maxPeaks.reduce(max) * 10;
      double barHeight = value * deltaTime / 5;
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
      isPlaying;
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
