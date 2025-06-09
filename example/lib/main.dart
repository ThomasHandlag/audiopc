import 'dart:async';

import 'package:audiopc/audio_metadata.dart';
import 'package:audiopc/audiopc_state.dart';
import 'package:audiopc/widgets/visualizer.dart';
import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';
import 'package:audiopc/audiopc.dart';
import 'package:audiopc/widgets/play_button.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();

  final player = Audiopc(id: "0");

  runApp(
    MaterialApp(
      theme: ThemeData.dark(useMaterial3: true),
      debugShowCheckedModeBanner: false,
      home: MyApp(player: player,),
    ),
  );
}

class MyApp extends StatefulWidget {
  const MyApp({super.key, required this.player});
  final Audiopc player;
  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> with SingleTickerProviderStateMixin {

  late AnimationController _controller;
  late Animation<double> _animation;

  int timeSec = 100;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: Duration(milliseconds: timeSec),
      animationBehavior: AnimationBehavior.preserve,
    );
    _controller.repeat(reverse: true);
    _animation = Tween<double>(begin: 0, end: 1).animate(_controller);

    widget.player.onCompleted.listen((event) {
      if (event) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text("Audio playback completed")),
        );
      }
    });

    widget.player.onStateChanged.listen((state) {
      setState(() {
        _isPlaying = state == PlayerState.playing;
      });
    });
  }

  bool _isPlaying = false;

  @override
  void dispose() {
    _controller.dispose();
    widget.player.dispose();
    super.dispose();
  }

  double rate = 1.0;

  String sPath = "";

  AudioMetaData? audioMetaData;

  Future<void> getMetaData(String path) async {
    final data = await widget.player.getMetadata(path);
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
                      openFile(
                        acceptedTypeGroups: <XTypeGroup>[typeGroup],
                      ).then((file) {
                        if (file != null) {
                          widget.player.play(file.path);
                          getMetaData(file.path);
                          setState(() {
                            sPath = file.path;
                          });
                        }
                      });
                    },
                    child: const Text("Choose file"),
                  ),
                  ElevatedButton(
                    onPressed: () {
                      setState(() {
                        rate += 0.1;
                      });
                      widget.player.setRate(rate);
                    },
                    child: Text("Rate+: $rate"),
                  ),
                  ElevatedButton(
                    onPressed: () {
                      setState(() {
                        rate -= 0.1;
                      });
                      widget.player.setRate(rate);
                    },
                    child: Text("Rate-: $rate"),
                  ),
                  PlayButton(
                    onPressed: () {
                      if (widget.player.isPlaying) {
                        widget.player.pause();
                      } else {
                        widget.player.resume();
                      }
                    },
                    child: Icon(_isPlaying ? Icons.pause : Icons.play_arrow),
                  ),
                  AudiopcSlider(
                    duration: widget.player.duration,
                    onPositionChanged: widget.player.onPositionChanged,
                    seek: (v) {
                      widget.player.seek(v);
                    },
                  ),
                  Text("Duration: ${widget.player.duration}"),
                  IconButton(
                    onPressed: () {
                      setState(() {
                        timeSec += 10;
                        _controller.duration = Duration(milliseconds: timeSec);
                      });
                    },
                    icon: const Icon(Icons.plus_one),
                  ),
                  IconButton(
                    onPressed: () {
                      setState(() {
                        timeSec -= 10;
                        _controller.duration = Duration(milliseconds: timeSec);
                      });
                    },
                    icon: const Icon(Icons.exposure_minus_1),
                  ),
                  Text("Time: $timeSec"),
                  StreamBuilder(
                    stream: widget.player.onSamples,
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
                              isPlaying: widget.player.isPlaying,
                            ),
                            size: Size(
                              MediaQuery.of(context).size.width * 0.8,
                              200,
                            ),
                          );
                        },
                      );
                    },
                  ),
                  StreamBuilder(
                    stream: widget.player.onSamples.asBroadcastStream(),
                    initialData: <double>[],
                    builder: (context, snapshot) {
                      return AnimatedBuilder(
                        animation: _animation,
                        builder: (_, __) {
                          return CustomPaint(
                            painter: CircleAudioVisualizerPainter(
                              _animation.value,
                              snapshot.data ?? [],
                              widget.player.isPlaying,
                              0,
                              64,
                            ),
                            size: Size(
                              MediaQuery.of(context).size.width * 0.8,
                              200,
                            ),
                            child: SizedBox(
                              width: MediaQuery.of(context).size.width * 0.5,
                              height: MediaQuery.of(context).size.width * 0.5,
                            ),
                          );
                        },
                      );
                    },
                  ),
                ],
              ),
            ),
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
            ),
          ),
        ],
      ),
    );
  }
}
