import 'dart:async';
import 'dart:js_interop';
import 'dart:typed_data';

import 'package:audiopc/state.dart';
import 'package:web/web.dart' as web;

class AudioBackendInfo {
  final int defaultOutputSampleRate;
  final int defaultOutputChannels;
  final int outputDeviceCount;

  const AudioBackendInfo({
    required this.defaultOutputSampleRate,
    required this.defaultOutputChannels,
    required this.outputDeviceCount,
  });

  bool get isAvailable =>
      defaultOutputSampleRate > 0 &&
      defaultOutputChannels > 0 &&
      outputDeviceCount >= 0;
}

AudioBackendInfo getAudioBackendInfo() {
  final context = web.AudioContext();
  final sampleRate = context.sampleRate.toInt();
  context.close();
  return AudioBackendInfo(
    defaultOutputSampleRate: sampleRate,
    defaultOutputChannels: 2,
    outputDeviceCount: 1,
  );
}

class AudiopcPlayer with StateMixin {
  late final web.AudioContext _audioContext;
  late final web.GainNode _gainNode;
  late final web.AnalyserNode _analyser;
  web.AudioBuffer? _decodedBuffer;
  web.AudioBufferSourceNode? _currentSource;
  bool _isPlaying = false;
  bool _sourceStopRequested = false;

  double _playbackOffsetSec = 0.0;
  double _playbackStartedAtCtxSec = 0.0;
  double _durationSec = 0.0;

  final StreamController<int> positionStreamController =
      StreamController<int>.broadcast();

  late final Timer _positionTimer;

  AudiopcPlayer() {
    _audioContext = web.AudioContext();
    _gainNode = _audioContext.createGain();
    _analyser = _audioContext.createAnalyser();
    _analyser.fftSize = 2048;

    _gainNode.connect(_analyser);
    _analyser.connect(_audioContext.destination);

    _positionTimer = Timer.periodic(const Duration(milliseconds: 100), (_) {
      if (_isPlaying && _durationSec > 0 && _positionSeconds >= _durationSec) {
        _isPlaying = false;
        _playbackOffsetSec = _durationSec;
        _disconnectCurrentSource();
        setState(PlayerState.idle);
      }
      positionStreamController.add(positionMillis);
    });
  }

  void _disconnectCurrentSource() {
    if (_currentSource == null) {
      return;
    }

    try {
      _currentSource!.disconnect();
    } catch (_) {
      // No-op for already-disconnected node.
    }
    _currentSource = null;
  }

  void _stopCurrentSource() {
    if (_currentSource == null) {
      return;
    }

    _sourceStopRequested = true;
    try {
      _currentSource!.stop();
    } catch (_) {
      // No-op for already-stopped node.
    }
    _disconnectCurrentSource();
  }

  double get _positionSeconds {
    if (_isPlaying) {
      final elapsed = _audioContext.currentTime - _playbackStartedAtCtxSec;
      return (_playbackOffsetSec + elapsed).clamp(0.0, _durationSec);
    }
    return _playbackOffsetSec.clamp(0.0, _durationSec);
  }

  void _startSourceAt(double offsetSec) {
    if (_decodedBuffer == null) {
      return;
    }

    _stopCurrentSource();

    final source = _audioContext.createBufferSource();
    source.buffer = _decodedBuffer;
    source.connect(_gainNode);
    source.onended = ((web.Event _) {
      if (_sourceStopRequested) {
        _sourceStopRequested = false;
        return;
      }
      _isPlaying = false;
      _playbackOffsetSec = _durationSec;
      _disconnectCurrentSource();
      setState(PlayerState.idle);
    }).toJS;

    source.start(0, offsetSec);
    _currentSource = source;
    _playbackOffsetSec = offsetSec;
    _playbackStartedAtCtxSec = _audioContext.currentTime;
    _isPlaying = true;
  }

  bool setFileSource(String path) {
    unawaited(_fetchAndDecodeAudio(path));
    return true;
  }

  bool setUrlSource(String url) {
    unawaited(_fetchAndDecodeAudio(url));
    return true;
  }

  bool setMemorySource(List<int> data) {
    // Web backend currently expects URL/path-based sources.
    return false;
  }

  Future<void> _fetchAndDecodeAudio(String source) async {
    setState(PlayerState.loading);
    try {
      final request = web.Request(source.toJS);
      final response = await web.window.fetch(request).toDart;
      if (!response.ok) {
        setState(PlayerState.error);
        return;
      }

      final arrayBuffer = await response.arrayBuffer().toDart;
      final decoded = await _audioContext.decodeAudioData(arrayBuffer).toDart;

      _decodedBuffer = decoded;
      _durationSec = decoded.duration;
      _playbackOffsetSec = 0.0;
      _isPlaying = false;
      _stopCurrentSource();
      setState(PlayerState.idle);
    } catch (e) {
      setState(PlayerState.error);
    }
  }

  bool play() {
    if (_decodedBuffer == null) {
      return false;
    }

    if (_audioContext.state == 'suspended') {
      unawaited(_audioContext.resume().toDart);
    }

    if (_isPlaying) {
      return true;
    }

    _startSourceAt(_playbackOffsetSec);
    setState(PlayerState.playing);
    return true;
  }

  bool pause() {
    if (!_isPlaying) {
      return true;
    }

    _playbackOffsetSec = _positionSeconds;
    _isPlaying = false;
    _stopCurrentSource();
    unawaited(_audioContext.suspend().toDart);
    setState(PlayerState.paused);
    return true;
  }

  bool stop() {
    _isPlaying = false;
    _playbackOffsetSec = 0.0;
    _stopCurrentSource();
    setState(PlayerState.idle);
    return true;
  }

  bool setVolume(double value) {
    _gainNode.gain.value = value.clamp(0.0, 4.0);
    return true;
  }

  bool setLowPassHz(double hz) {
    // Placeholder on web backend for API compatibility.
    return true;
  }

  int get bufferedSamples => 0; // Web Audio doesn't expose this
  int get positionMillis => (_positionSeconds * 1000).toInt();
  int get durationMillis => (_durationSec * 1000).toInt();
  int get visualizerAvailableSamples => _analyser.frequencyBinCount;
  int get visualizerSampleRate => _audioContext.sampleRate.toInt();
  int get visualizerChannels => 1; // Analyser output

  List<double> getVisualizerSamples(int maxSamples) {
    if (maxSamples <= 0) {
      return const [];
    }

    final data = Uint8List(_analyser.frequencyBinCount);
    _analyser.getByteFrequencyData(data.toJS);
    final count = maxSamples.clamp(0, data.length.toInt());

    final out = <double>[];
    for (var i = 0; i < count; i++) {
      out.add(data[i].toDouble() / 255.0);
    }
    return out;
  }

  List<double> getVisualizerSpectrum(int barCount) {
    if (barCount <= 0) {
      return const [];
    }

    final data = Uint8List(_analyser.frequencyBinCount);
    _analyser.getByteFrequencyData(data.toJS);

    final result = <double>[];
    final step = data.length.toInt() / barCount;
    for (var i = 0; i < barCount; i++) {
      final idx = (i * step).toInt();
      if (idx < data.length.toInt()) {
        result.add(data[idx].toDouble() / 255.0);
      } else {
        result.add(0.0);
      }
    }
    return result;
  }

  void seek(int positionMillis) {
    if (_decodedBuffer == null) {
      return;
    }

    final targetSec = (positionMillis / 1000.0).clamp(0.0, _durationSec);
    final wasPlaying = _isPlaying;

    _playbackOffsetSec = targetSec;
    if (wasPlaying) {
      _startSourceAt(targetSec);
      setState(PlayerState.playing);
    }
  }

  void dispose() {
    _isPlaying = false;
    _stopCurrentSource();
    _positionTimer.cancel();
    positionStreamController.close();
    stateStreamController.close();
    unawaited(_audioContext.close().toDart);
  }
}
