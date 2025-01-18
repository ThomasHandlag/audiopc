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

    final stft = STFT(data.length, Window.hanning(data.length));
    final spectrogram = <Float64List>[];
    stft.run(data, (Float64x2List freq) {
      spectrogram.add(freq.discardConjugates().magnitudes());
    });

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


  List<double> getPeaks(List<double> data, int barCount) {
    final stft = STFT(2048, Window.hanning(2048));
    final spectrogram = <Float64List>[];
    final freqBin = List.generate(128, (i) => 44100 / 2 * i / 128);

    // final freqBin = List.generate(128, (i) {
    //   double t = i / 128;
    //   return 20.0 * pow(20000.0 / 20.0, t);
    // });
    
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
            double smoothedValue = max(
              magnitude,
              lastSpectrum[i] * fitFactor
            );
            lastSpectrum[i] = smoothedValue;
          }
        }
      }
    }
    return lastSpectrum;    
  }
 
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
