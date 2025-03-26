part of '../audiopc.dart';

class AudiopcSlider extends StatefulWidget {
  final Stream<double> onPositionChanged;
  final double duration;

  const AudiopcSlider(
      {super.key,
      required this.duration,
      required this.onPositionChanged,
      required this.seek});

  final void Function(double) seek;

  @override
  State<AudiopcSlider> createState() => _AudiopcSliderState();
}

class _AudiopcSliderState extends State<AudiopcSlider> {
  bool isDragging = false;

  double value = 0;

  @override
  Widget build(BuildContext context) {
    return StreamBuilder(
        stream: widget.onPositionChanged,
        builder: (_, snapshot) {
          return Slider(
            value: !isDragging ? snapshot.data ?? 0 : value,
            onChanged: (v) {
              setState(() {
                value = v;
              });
            },
            onChangeStart: (v) {
              setState(() {
                isDragging = true;
              });
            },
            onChangeEnd: (v) {
              setState(() {
                isDragging = false;
              });
              if (!isDragging) {
                widget.seek(value);
              }
            },
            max: widget.duration + 1,
          );
        });
  }
}
