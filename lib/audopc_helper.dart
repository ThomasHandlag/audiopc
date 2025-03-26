import 'dart:async';

import 'package:flutter/scheduler.dart';

abstract class PlayerListener {
  void stop();
  void start();
  void pause();
  Future<void> call();
}

abstract class FrameCallBackUpdater {
  FrameCallBackUpdater();

  /// callback function to be called when the frame is ready
  /// [timeStamp] is the time of each frame
  void callback(Duration? timeStamp);
}

final class PositionListener extends PlayerListener
    implements FrameCallBackUpdater {
  PositionListener({required this.getPosition});
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
    callback(null);
  }

  @override
  void callback(Duration? timeStamp) {
    if (isRunnin) {
      SchedulerBinding.instance.scheduleFrameCallback(callback);
      call();
    }
  }

  @override
  void stop() {
    isRunnin = false;
    SchedulerBinding.instance.cancelFrameCallbackWithId(0);
    streamControler.close();
  }

  bool isRunnin = false;

  @override
  void pause() {
    isRunnin = false;
  }
}

final class SamplesListener extends PlayerListener
    implements FrameCallBackUpdater {
  SamplesListener({required this.getSamples}) {
    streamControler.onCancel = stop;
  }

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
    isRunnin = false;
    SchedulerBinding.instance.cancelFrameCallbackWithId(0);
    streamControler.close();
  }

  @override
  void start() {
    isRunnin = true;
    callback(null);
  }

  @override
  void callback(Duration? timeStamp) {
    if (isRunnin) {
      SchedulerBinding.instance.scheduleFrameCallback(callback);
      call();
    }
  }

  @override
  void pause() {
    isRunnin = false;
  }
}
