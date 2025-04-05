import 'package:audiopc/audio_metadata.dart';
import 'package:audiopc/widgets/visualizer.dart';
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
    _audiopcPlugin.onDurationChanged.listen(null);

    _audiopcPlugin.onCompleted.listen((val) {
      if (val) {
        ScaffoldMessenger.of(context)
        .showSnackBar(const SnackBar(content: Text("Completed")));
      }
    });
  }

  double rate = 1.0;

  String sPath = "";

  AudioMetaData? audioMetaData;

  Future<void> getMetaData(String path) async {
    final data = await _audiopcPlugin.getMetadata(path);
    setState(() {
      audioMetaData = data;
    });
  }

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
                              getMetaData(file.path);
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
                        icon: StreamBuilder(
                            stream: _audiopcPlugin.onStateChanged,
                            builder: (_, snapshot) {
                              return Icon(snapshot.data == AudiopcState.playing
                                  ? Icons.pause
                                  : Icons.play_arrow);
                            })),
                    StreamBuilder(
                        stream: _audiopcPlugin.onDurationChanged,
                        builder: (_, val) {
                          return AudiopcSlider(
                              duration: val.data ?? 0,
                              onPositionChanged:
                                  _audiopcPlugin.onPositionChanged,
                              seek: (v) {
                                _audiopcPlugin.seek(v);
                              });
                        }),
                    StreamBuilder(
                        stream: _audiopcPlugin.onDurationChanged,
                        builder: (_, val) {
                          return Text("Duration: ${val.data ?? 0}");
                        }),
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
                  audioMetaData?.title != null
                      ? Text(
                          "Title: ${audioMetaData?.title}",
                          overflow: TextOverflow.ellipsis,
                        )
                      : const SizedBox(),
                  Text("Artist: ${audioMetaData?.artist}"),
                  if (audioMetaData != null && audioMetaData!.thumbnail != null)
                    Image.memory(
                      audioMetaData!.thumbnail!,
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
