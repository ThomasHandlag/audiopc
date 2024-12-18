import 'package:flutter/material.dart';
import 'dart:async';

import 'package:audiopc/audiopc.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatefulWidget {
  const MyApp({super.key});

  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  final _audiopcPlugin = Audiopc();

  double _duration = 0.0;
  double _cDuration = 0.0;

  Future<void> _getDuration() async {
    final double? duration = await _audiopcPlugin.getDuration();

    if (!mounted) return;

    setState(() {
      _duration = duration!;
    });
  }

  Future<void> _getCDuration() async {
    final double? cDuration = await _audiopcPlugin.getCurrentPosition();

    if (!mounted) return;

    setState(() {
      _cDuration = cDuration!;
    });
  }

  @override
  void initState() {
    super.initState();
    Timer.periodic(const Duration(milliseconds: 10), (timer) {
      if (isPlaying) {
        _getCDuration();
      }
    });
  }

  bool isPlaying = false;

  Song? _character = Song.nodoubt;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
        home: Scaffold(
      appBar: AppBar(
        title: const Text('Audiopc example app'),
      ),
      body: Center(
        child: Column(
          children: [
            ListTile(
              title: const Text('No doubt'),
              leading: Radio<Song>(
                value: Song.nodoubt,
                groupValue: _character,
                onChanged: (Song? value) {
                  setState(() {
                    _character = value;
                  });
                },
              ),
            ),
            ListTile(
              title: const Text('The hype'),
              leading: Radio<Song>(
                value: Song.thehype,
                groupValue: _character,
                onChanged: (Song? value) {
                  setState(() {
                    _character = value;
                  });
                },
              ),
            ),
            IconButton(
                onPressed: () {
                  _audiopcPlugin.setSource(_character == Song.nodoubt
                      ? 'D:/Downloads/mine.mp3'
                      : 'D:/Downloads/goagain.mp3');
                },
                icon: const Icon(Icons.source_outlined)),
            IconButton(
                onPressed: () {
                  if (isPlaying) {
                    _audiopcPlugin.pause();
                    setState(() {
                      isPlaying = false;
                    });
                  } else {
                    _audiopcPlugin.play();
                    setState(() {
                      isPlaying = true;
                    });
                  }
                  _getDuration();
                },
                icon: Icon(isPlaying ? Icons.pause : Icons.play_arrow)),
            Slider(
              value: _cDuration,
              onChanged: (value) {
                _audiopcPlugin.seek(value);
              },
              max: _duration + 1,
            ),
            Text("$_duration"),
          ],
        ),
      ),
    ));
  }
}

enum Song { nodoubt, thehype }
