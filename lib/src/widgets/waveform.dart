import 'package:flutter/material.dart';

class WaveformControl extends StatelessWidget {
  const WaveformControl({
    super.key,
    required this.onChanged,
    required this.value,
    required this.data,
  });

  final ValueChanged<double> onChanged;
  final double value;
  final List<double> data;

  void _updatePosition(Offset localPosition, double width) {
    if (width <= 0) return;

    onChanged((localPosition.dx / width).clamp(0.0, 1.0));
  }

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final width = constraints.maxWidth;

        return GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTapDown: (details) {
            _updatePosition(details.localPosition, width);
          },
          onHorizontalDragStart: (details) {
            _updatePosition(details.localPosition, width);
          },
          onHorizontalDragUpdate: (details) {
            _updatePosition(details.localPosition, width);
          },
          child: SizedBox(
            width: width,
            height: 64,
            child: CustomPaint(
              painter: WaveformPainter(
                data: data,
                value: value.clamp(0.0, 1.0),
                activeColor: Theme.of(context).colorScheme.primary,
                inactiveColor: Theme.of(
                  context,
                ).colorScheme.onSurface.withAlpha(45),
                gap: 2,
              ),
            ),
          ),
        );
      },
    );
  }
}

class WaveformPainter extends CustomPainter {
  const WaveformPainter({
    required this.data,
    required this.value,
    required this.activeColor,
    required this.inactiveColor,
    this.gap = 2,
  });

  final List<double> data;
  final double value;
  final Color activeColor;
  final Color inactiveColor;
  final double gap;

  @override
  void paint(Canvas canvas, Size size) {
    if (data.isEmpty || size.isEmpty) return;

    final count = data.length;
    final totalGap = gap * (count - 1);
    final barWidth = (size.width - totalGap) / count;
    final centerY = size.height / 2;
    final progressX = size.width * value.clamp(0.0, 1.0);

    final activePaint = Paint()..color = activeColor;
    final inactivePaint = Paint()..color = inactiveColor;
    final radius = Radius.circular(barWidth / 2);

    // Draw all waveform bars in the inactive color.
    for (var i = 0; i < count; i++) {
      final x = i * (barWidth + gap);
      final amplitude = data[i].clamp(0.0, 1.0);
      final barHeight = (size.height * amplitude).clamp(3.0, size.height);

      final bar = RRect.fromRectAndRadius(
        Rect.fromLTWH(x, centerY - barHeight / 2, barWidth, barHeight),
        radius,
      );

      canvas.drawRRect(bar, inactivePaint);
    }

    // Reveal the active waveform continuously up to the progress.
    canvas.save();
    canvas.clipRect(Rect.fromLTWH(0, 0, progressX, size.height));

    for (var i = 0; i < count; i++) {
      final x = i * (barWidth + gap);
      final amplitude = data[i].clamp(0.0, 1.0);
      final barHeight = (size.height * amplitude).clamp(3.0, size.height);

      final bar = RRect.fromRectAndRadius(
        Rect.fromLTWH(x, centerY - barHeight / 2, barWidth, barHeight),
        radius,
      );

      canvas.drawRRect(bar, activePaint);
    }

    canvas.restore();
  }

  @override
  bool shouldRepaint(covariant WaveformPainter oldDelegate) {
    return oldDelegate.value != value ||
        oldDelegate.data != data ||
        oldDelegate.activeColor != activeColor ||
        oldDelegate.inactiveColor != inactiveColor ||
        oldDelegate.gap != gap;
  }
}
