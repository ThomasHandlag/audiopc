import 'package:audiopc/audiopc.dart';
import 'package:flutter/material.dart';

class FilterControls extends StatefulWidget {
  const FilterControls({super.key, required this.player});

  final AudioPlayer player;

  @override
  State<FilterControls> createState() => _FilterControlsState();
}

class _FilterControlsState extends State<FilterControls> {
  AudioPlayer get player => widget.player;

  double peak = 1000;
  double lowShelf = 100;
  double highShelf = 5000;
  double bandPass = 1000;
  double notch = 1000;
  double q = 0.5;
  double gainDB = 10;
  double comb = 150;
  double combFeedback = 2;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text('Filters'),
        const SizedBox(height: 8),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            const Text('Peak'),
            Slider(
              value: peak,
              onChanged: (value) {
                setState(() {
                  peak = value.clamp(10, 20000);
                });

                player.setPeakFilter(peak, q, gainDB);
              },
              divisions: 10,
              label: 'Peak frequency',
              max: 20000,
              min: 10
            ),
            const Text('Low Shelving'),
            Slider(
              value: lowShelf,
              onChanged: (value) {
                setState(() {
                  lowShelf = value.clamp(10, 20000);
                });
                player.setLowShelfFilter(lowShelf, q, gainDB);
              },
              divisions: 100,
              label: 'Low shelf frequency',
              max: 20000,
            ),
            const Text('High Shelving'),
            Slider(
              value: highShelf,
              onChanged: (value) {
                setState(() {
                  highShelf = value.clamp(10, 20000);
                });
                player.setHighShelfFilter(highShelf, q, gainDB);
              },
              label: 'High shelf frequency',
              max: 20000,
              divisions: 100,
            ),
            const Text('Band-pass'),
            Slider(
              value: bandPass,
              onChanged: (value) {
                setState(() {
                  bandPass = value.clamp(10, 20000);
                });

                player.setBandPassFilter(bandPass, q);
              },
              label: 'Band-pass frequency',
              divisions: 100,
              max: 20000,
            ),
            const Text('Notch'),
            Slider(
              value: notch,
              onChanged: (value) {
                setState(() {
                  notch = value.clamp(10, 20000);
                });
                player.setNotchFilter(notch, q, 0);
              },
              label: 'Notch frequency',
              divisions: 100,
              max: 20000,
            ),
            const Text('Q factor'),
            Slider(
              value: q,
              onChanged: (value) {
                setState(() {
                  q = value.clamp(0.1, 10);
                });
              },
              label: 'Q factor',
              min: 0.1,
              max: 10,
            ),
            const Text('Gain (dB)'),
            Slider(
              value: gainDB,
              onChanged: (value) {
                setState(() {
                  gainDB = value.clamp(-30, 30);
                });
              },
              label: 'Gain',
              max: 30,
              divisions: 100,
              min: .1,
            ),
            ElevatedButton(
              onPressed: player.clearFilter,
              child: const Text('Clear filters'),
            ),
          ],
        ),
      ],
    );
  }
}
