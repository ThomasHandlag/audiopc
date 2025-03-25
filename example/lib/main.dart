import 'package:audiopc/audio_metadata.dart';
import 'package:audiopc_example/visualizer.dart';
import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';
import 'package:audiopc/audiopc.dart';
import 'package:audiopc/audiopc_state.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(MaterialApp(
      theme: ThemeData.dark(useMaterial3: true),
      debugShowCheckedModeBanner: false,
      home: MyApp()));
}

class MyApp extends StatefulWidget {
  const MyApp({super.key});

  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> with SingleTickerProviderStateMixin {
  final _audiopcPlugin = Audiopc(id: "0");

  double _duration = 0.0;
  double _cDuration = 0.0;

  // final CircularBuffer<double> _sampleBuffer = CircularBuffer(max: 88900);
  List<double> _sampleBuffer = [];

  late AnimationController _controller;
  late Animation<double> _animation;

  int timeSec = 100;

  @override
  void initState() {
    super.initState();

    _controller = AnimationController(
        vsync: this,
        duration: Duration(milliseconds: timeSec),
        animationBehavior: AnimationBehavior.preserve);

    _controller.addStatusListener((status) {
      if (status == AnimationStatus.completed) {
        _controller.reverse();
      } else if (status == AnimationStatus.dismissed) {
        _controller.forward();
      }
    });

    _controller.forward();
    _audiopcPlugin.onDurationChanged.listen((event) {
      setState(() {
        _duration = event;
      });
    });

    _animation = Tween<double>(begin: 0, end: 1).animate(_controller);

    _audiopcPlugin.onPositionChanged.listen((position) {
      setState(() {
        _cDuration = position;
      });
    });

    _audiopcPlugin.onStateChanged.listen((state) {
      if (state == AudiopcState.playing) {
        setState(() {
          isPlaying = true;
        });
      } else {
        setState(() {
          isPlaying = false;
        });
      }
    });

    _audiopcPlugin.onSamples.listen((samples) {
      setState(() {
        _sampleBuffer = samples;
      });
    });

    _audiopcPlugin.onCompleted.listen((completed) {
      if (completed) {
       debugPrint("Completed");
      }
    });
  }

  double rate = 1.0;

  bool isPlaying = false;

  String sPath = "";

  AudioMetaData? snapshot;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Row(
        children: [
          SingleChildScrollView(
            child: SizedBox(
                width: MediaQuery.of(context).size.width * 0.5,
                child: Column(
                  children: [
                    ElevatedButton(
                        onPressed: () {
                          const XTypeGroup typeGroup = XTypeGroup(
                            label: 'audio',
                            extensions: <String>['mp3', 'wav', 'flac'],
                          );
                          openFile(acceptedTypeGroups: <XTypeGroup>[typeGroup])
                              .then((file) {
                            if (file != null) {
                              _audiopcPlugin.play(file.path);
                              _audiopcPlugin
                                  .getMetadata(file.path)
                                  .then((value) {
                                setState(() {
                                  snapshot = value;
                                });
                              });

                              setState(() {
                                sPath = file.path;
                              });
                            }
                          });
                        },
                        child: const Text("Choose file")),
                    ElevatedButton(
                        onPressed: () {
                          setState(() {
                            rate += 0.1;
                          });
                          _audiopcPlugin.setRate(rate);
                        },
                        child: Text("Rate+: $rate")),
                    ElevatedButton(
                        onPressed: () {
                          setState(() {
                            rate -= 0.1;
                          });
                          _audiopcPlugin.setRate(rate);
                        },
                        child: Text("Rate-: $rate")),
                    IconButton(
                        onPressed: () {
                          if (isPlaying) {
                            _audiopcPlugin.pause();
                          } else {
                            _audiopcPlugin.resume();
                          }
                        },
                        icon: Icon(isPlaying ? Icons.pause : Icons.play_arrow)),
                    Slider(
                      value:
                          _audiopcPlugin.duration < _cDuration ? 0 : _cDuration,
                      onChanged: (value) {
                        _audiopcPlugin.seek(value);
                      },
                      max: _duration + 1,
                    ),
                    Text("$_duration"),
                    IconButton(
                        onPressed: () {
                          setState(() {
                            timeSec += 10;
                            _controller.duration =
                                Duration(milliseconds: timeSec);
                          });
                        },
                        icon: const Icon(Icons.plus_one)),
                    IconButton(
                        onPressed: () {
                          setState(() {
                            timeSec -= 10;
                            _controller.duration =
                                Duration(milliseconds: timeSec);
                          });
                        },
                        icon: const Icon(Icons.exposure_minus_1)),
                    Text("Time: $timeSec"),
                    AnimatedBuilder(
                        animation: _animation,
                        builder: (_, __) {
                          return CustomPaint(
                            painter: VisualzerPainter(
                                clipper: const VisualizerClipper(),
                                deltaTime: _controller.value,
                                data: _sampleBuffer,
                                isPlaying: isPlaying),
                            size: Size(
                                MediaQuery.of(context).size.width * 0.8, 200),
                          );
                        }),
                    AnimatedBuilder(
                        animation: _animation,
                        builder: (_, __) {
                          return CustomPaint(
                            painter: CircleAudioVisualizerPainter(
                                _animation.value,
                                _sampleBuffer,
                                isPlaying,
                                0,
                                64),
                            size: Size(
                                MediaQuery.of(context).size.width * 0.8, 200),
                            child: SizedBox(
                              width: MediaQuery.of(context).size.width * 0.5,
                              height: MediaQuery.of(context).size.width * 0.5,
                            ),
                          );
                        }),
                  ],
                )),
          ),
          SizedBox(
              width: MediaQuery.of(context).size.width * 0.3,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.center,
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  snapshot?.title != null
                      ? Text(
                          "Title: ${snapshot?.title}",
                          overflow: TextOverflow.ellipsis,
                        )
                      : const SizedBox(),
                  Text("Artist: ${snapshot?.artist}"),
                  if (snapshot != null && snapshot!.thumbnail != null)
                    Image.memory(
                      snapshot!.thumbnail!,
                      width: 200,
                      height: 200,
                    ),
                ],
              ))
        ],
      ),
    );
  }
}
