import 'dart:async';

import 'package:flutter/scheduler.dart';

abstract class PlayerListener {
  void stop();
  void start();
  void pause();
  Future<void> call();
}

abstract class FrameCallBackUpdater {
  int id;
  FrameCallBackUpdater({required this.id});

  /// callback function to be called when the frame is ready
  /// [timeStamp] is the time of each frame
  void callback(Duration? timeStamp);
}

final class PositionListener extends PlayerListener
    implements FrameCallBackUpdater {
  PositionListener({required this.getPosition, required this.id});
  final Future<double> Function() getPosition;
  final streamControler = StreamController<double>.broadcast();

  Stream<double> get positionStream => streamControler.stream;

  @override
  Future<void> call() async {
    final position = await getPosition();
    streamControler.add(position);
  }

  @override
  void start() {
    isRunnin = true;
  }

  @override
  void callback(Duration? timeStamp) {
    if (isRunnin) {
      call();
      SchedulerBinding.instance.scheduleFrameCallback(callback);
    }
  }

  @override
  void stop() {
    isRunnin = false;
    streamControler.close();
    SchedulerBinding.instance.cancelFrameCallbackWithId(id);
  }

  @override
  int id;

  bool isRunnin = false;

  @override
  void pause() {
    isRunnin = false;
  }
}

final class SamplesListener extends PlayerListener
    implements FrameCallBackUpdater {
  SamplesListener({required this.getSamples, required this.id});

  final Future<List<double>> Function() getSamples;
  final streamControler = StreamController<List<double>>.broadcast();
  Stream<List<double>> get samplesStream => streamControler.stream;

  bool isRunnin = false;

  @override
  Future<void> call() async {
    final samples = await getSamples();
    streamControler.add(samples);
  }

  @override
  void stop() {
    streamControler.close();
  }

  @override
  void start() {
    // do nothing
  }

  @override
  int id;

  @override
  void callback(Duration? timeStamp) {
    if (isRunnin) {
      call();
      SchedulerBinding.instance.scheduleFrameCallback(callback);
    }
  }

  @override
  void pause() {
    isRunnin = false;
  }
}
