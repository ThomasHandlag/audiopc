import 'package:audiopc/audio_metadata.dart';
import 'package:audiopc_example/visualizer.dart';
import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';
import 'package:audiopc/audiopc.dart';
import 'package:audiopc/audiopc_state.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const MyApp());
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

  final Map<String, String> songs = {
    "flyagain": "D:/Downloads/flyagain.mp3",
    "gravity": "D:/Downloads/gravity.mp3",
    "holdon": "D:/Downloads/holdon.mp3",
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
    "oblivion": "D:/Downloads/oblivion.mp3",
    "ifeelcrazy": "D:/Downloads/ifeelcrazy.mp3",
    "getouthere": "D:/Downloads/getouthere.mp3",
    "vessel": "D:/Downloads/vessel.mp3",
    "vietnam": "D:/Downloads/vietnam.mp3",
    "forever": "D:/Downloads/forever.mp3",
    "runaway": "D:/Downloads/runaway.mp3",
    "loveu": "D:/Downloads/loveu.mp3",
    "survive": "D:/Downloads/survive.mp3",
    "calling": "D:/Downloads/calling.mp3",
    "whydoi": "D:/Downloads/whydoi.mp3",
    "lemonstronger": "D:/Downloads/lemonstronger.mp3",
    "alwaysbe": "D:/Downloads/alwaysbe.mp3",
    "nodoubt": "D:/Downloads/nodoubt.mp3",
    "thistime": "D:/Downloads/thistime.mp3",
    "mushoku": "D:/Downloads/mushoku.mp3",
    "safensound": "D:/Downloads/safensound.mp3",
  };

  final CircularBuffer<double> _sampleBuffer = CircularBuffer(max: 44100);


  late AnimationController _controller;

  @override
  void initState() {
    super.initState(); 

    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 90),
    );

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
         _sampleBuffer.addAll(samples);
      });
    });

    _audiopcPlugin.onCompleted.listen((completed) {
      if (completed) {
        debugPrint("completed");
      }
    });
  }

  double rate = 1.0;

  bool isPlaying = false;

  String sPath = "";

  AudioMetaData? snapshot;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
        theme: ThemeData.dark(useMaterial3: true),
        home: Scaffold(
          appBar: AppBar(
            title: const Text("Audiopc example"),
          ),
          body: Row(
            children: [
              MediaQuery.of(context).size.width < 569
                  ? ElevatedButton(
                      onPressed: () {
                        const XTypeGroup typeGroup = XTypeGroup(
                          label: 'audio',
                          extensions: <String>['mp3', 'wav', 'flac'],
                        );
                        openFile(acceptedTypeGroups: <XTypeGroup>[typeGroup])
                            .then((file) {
                          if (file != null) {
                            _audiopcPlugin.play(file.path);
                            setState(() {
                              sPath = file.path;
                            });
                          }
                        });
                      },
                      child: const Text("Choose file"))
                  : SizedBox(
                      width: MediaQuery.of(context).size.width * 0.2,
                      child: ListView.builder(
                          itemCount: songs.length,
                          itemBuilder: (context, index) {
                            return ListTile(
                              title: Text(songs.keys.elementAt(index)),
                              onTap: () {
                                _audiopcPlugin
                                    .play(songs.values.elementAt(index));
                                setState(() {
                                  sPath = songs.values.elementAt(index);
                                });
                                _audiopcPlugin.getMetadata(sPath).then(
                                  (value) {
                                    setState(() {
                                      snapshot = value;
                                    });
                                    debugPrint(snapshot.toString());
                                  },
                                );
                              },
                            );
                          })),
              SingleChildScrollView(
                child: SizedBox(
                    width: MediaQuery.of(context).size.width * 0.5,
                    child: Column(
                      children: [
                        ElevatedButton(
                            onPressed: () {
                              setState(() {
                                rate += 0.1;
                              });
                              _audiopcPlugin.setRate(rate);
                            },
                            child: Text("Rate: $rate")),
                        IconButton(
                            onPressed: () {
                              if (isPlaying) {
                                _audiopcPlugin.pause();
                              } else {
                                _audiopcPlugin.resume();
                              }
                            },
                            icon: Icon(
                                isPlaying ? Icons.pause : Icons.play_arrow)),
                        Slider(
                          value: _audiopcPlugin.duration < _cDuration
                              ? 0
                              : _cDuration,
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
                              data: _sampleBuffer.buffer,
                              isPlaying: isPlaying),
                          size: Size(
                              MediaQuery.of(context).size.width * 0.8, 200),
                        ),
                        CustomPaint(
                          painter: CircleAudioVisualizerPainter(
                              _controller.value, _sampleBuffer.buffer, isPlaying, 0, 64),
                          size: Size(
                              MediaQuery.of(context).size.width * 0.8, 200),
                          child: SizedBox(
                            width: MediaQuery.of(context).size.width * 0.5,
                            height: MediaQuery.of(context).size.width * 0.5,
                          ),
                        ),
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
                                ? Text("Title: ${snapshot?.title}", overflow: TextOverflow.ellipsis,)
                                : const SizedBox(),
                            Text("Artist: ${snapshot?.artist}"),
                            if (snapshot != null && snapshot!.thumbnail != null)
                              Image.memory(snapshot!.thumbnail!, width: 200, height: 200,),
                          ],
                        )
              )
            ],
          ),
        ));
  }
}

enum Song { nodoubt, thehype }
