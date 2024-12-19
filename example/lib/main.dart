import 'package:audiopc_example/visualizer.dart';
import 'package:flutter/material.dart';

import 'package:audiopc/audiopc.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatefulWidget {
  const MyApp({super.key});

  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> with SingleTickerProviderStateMixin {
  final _audiopcPlugin = Audiopc();

  double _duration = 0.0;
  double _cDuration = 0.0;

  final Map<String, String> songs = {
    "nodoubt": "D:/Downloads/nodoubt.mp3",
    "thehype": "D:/Downloads/thehype.mp3",
    "underrated": "D:/Downloads/underrated.mp3",
    "mine": "D:/Downloads/mine.mp3",
    "goagain": "D:/Downloads/goagain.mp3",
    "allforu": "D:/Downloads/allforyou.mp3",
    "tetdongday1": "D:/Downloads/tetdongday1.mp3",
    "tetdongday2": "D:/Downloads/tetdongday2.mp3",
    "tetdongday4": "D:/Downloads/tetdongday4.mp3",
    "lostinmiddle": "D:/Downloads/lostinmiddle.mp3",
    "stronger": "D:/Downloads/stronger.mp3",
    "stars": "D:/Downloads/stars.mp3",
    "makeumove": "D:/Downloads/makeumove.mp3",
  };

  List<double> data = [];

  late AnimationController _controller;

  @override
  void initState() {
    super.initState();

    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 100),
    );

    _controller.addStatusListener((status) {
      if (status == AnimationStatus.completed) {
        _controller.reverse();
      } else if (status == AnimationStatus.dismissed) {
        _controller.forward();
      }
    });

    _controller.forward();
    _audiopcPlugin.onDurationChanged = (duration) {
      setState(() {
        _duration = duration;
      });
    };

    _audiopcPlugin.onPositionChanged = (position) {
      setState(() {
        _cDuration = position;
      });
    };

    _audiopcPlugin.onStateChanged = (state) {
      if (state == AudiopcState.playing) {
        setState(() {
          isPlaying = true;
        });
      } else {
        setState(() {
          isPlaying = false;
        });
      }
    };

    _audiopcPlugin.onSamplesChanged = (samples) {
      setState(() {
        data = samples;
      });
    };
  }

  bool isPlaying = false;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
        home: Scaffold(
          backgroundColor: Colors.black,
      body: Row(
        children: [
          MediaQuery.of(context).size.width < 569
              ? ElevatedButton(
                  onPressed: () {}, child: const Text("Choose file"))
              : SizedBox(
                  width: MediaQuery.of(context).size.width * 0.2,
                  child: ListView.builder(
                      itemCount: songs.length,
                      itemBuilder: (context, index) {
                        return ListTile(
                          title: Text(songs.keys.elementAt(index)),
                          onTap: () {
                            _audiopcPlugin
                                .setSource(songs.values.elementAt(index));
                            _audiopcPlugin.play();
                          },
                        );
                      })),
          SingleChildScrollView(
            child: SizedBox(
              width: MediaQuery.of(context).size.width * 0.7,
                child: 
                    Column(
              children: [
                IconButton(
                    onPressed: () {
                      if (isPlaying) {
                        _audiopcPlugin.pause();
                      } else {
                        _audiopcPlugin.play();
                      }
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
                CustomPaint(
                  painter: VisualzerPainter(
                      clipper: const VisualizerClipper(),
                      deltaTime: _controller.value,
                      data: data,
                      isPlaying: isPlaying),
                  size: Size(MediaQuery.of(context).size.width * 0.8, 200),
                )
              ],
            )),
          ),
        ],
      ),
    ));
  }
}

enum Song { nodoubt, thehype }
