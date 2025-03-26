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

    _animation = Tween<double>(begin: 0, end: 1).animate(_controller);
  }

  double rate = 1.0;

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
                          if (_audiopcPlugin.state == AudiopcState.playing) {
                            _audiopcPlugin.pause();
                          } else {
                            _audiopcPlugin.resume();
                          }
                        },
                        icon: Icon(_audiopcPlugin.isPlaying
                            ? Icons.pause
                            : Icons.play_arrow)),
                    StreamBuilder(
                        stream: _audiopcPlugin.onPositionChanged,
                        builder: (_, snapshot) {
                          return Slider(
                            value: _audiopcPlugin.duration <
                                    _audiopcPlugin.position
                                ? 0
                                : snapshot.data ?? 0,
                            onChanged: (value) {
                              _audiopcPlugin.seek(value);
                            },
                            max: _audiopcPlugin.duration + 1,
                          );
                        }),
                    Text("${_audiopcPlugin.duration}"),
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
                    StreamBuilder(
                        stream: _audiopcPlugin.onSamples,
                        initialData: <double>[],
                        builder: (context, snapshot) {
                          return AnimatedBuilder(
                              animation: _animation,
                              builder: (_, __) {
                                return CustomPaint(
                                  painter: VisualzerPainter(
                                      clipper: const VisualizerClipper(),
                                      deltaTime: _controller.value,
                                      data: snapshot.data ?? [],
                                      isPlaying: _audiopcPlugin.isPlaying),
                                  size: Size(
                                      MediaQuery.of(context).size.width * 0.8,
                                      200),
                                );
                              });
                        }),
                    StreamBuilder(
                        stream: _audiopcPlugin.onSamples.asBroadcastStream(),
                        initialData: <double>[],
                        builder: (context, snapshot) {
                          return AnimatedBuilder(
                              animation: _animation,
                              builder: (_, __) {
                                return CustomPaint(
                                  painter: CircleAudioVisualizerPainter(
                                      _animation.value,
                                      snapshot.data ?? [],
                                      _audiopcPlugin.isPlaying,
                                      0,
                                      64),
                                  size: Size(
                                      MediaQuery.of(context).size.width * 0.8,
                                      200),
                                  child: SizedBox(
                                    width:
                                        MediaQuery.of(context).size.width * 0.5,
                                    height:
                                        MediaQuery.of(context).size.width * 0.5,
                                  ),
                                );
                              });
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
