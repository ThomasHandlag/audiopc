import 'dart:async';
import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';

/// A widget that renders a GLSL fragment shader with audio spectrum data
/// passed as a texture sampler (iChannel1) for real-time audio visualization.
class ShaderWidget extends StatefulWidget {
  /// The normalized spectrum data (0.0–1.0) used to drive the shader.
  final List<double> spectrum;

  final List<double> uniforms;
  final String shaderName;

  /// Width of the spectrum texture (number of frequency bins).
  final int spectrumWidth;

  const ShaderWidget({
    super.key,
    required this.spectrum,
    required this.uniforms,
    required this.shaderName,
    this.spectrumWidth = 64,
  });

  @override
  State<ShaderWidget> createState() => _ShaderWidgetState();
}

class _ShaderWidgetState extends State<ShaderWidget> {
  ui.FragmentProgram? _program;
  bool _programLoaded = false;
  int _frame = 0;
  Timer? _frameTimer;
  ui.Image? _spectrumTexture;

  @override
  void initState() {
    super.initState();
    _loadShader();
    _frameTimer = Timer.periodic(
      const Duration(milliseconds: 16), // ~60fps
      (_) {
        if (mounted) {
          setState(() {
            _frame++;
          });
        }
      },
    );
  }

  @override
  void didUpdateWidget(ShaderWidget oldWidget) {
    super.didUpdateWidget(oldWidget);
    // Regenerate texture if spectrum changed
    if (oldWidget.spectrum != widget.spectrum) {
      _updateTexture();
    }
  }

  @override
  void dispose() {
    _frameTimer?.cancel();
    _spectrumTexture?.dispose();
    super.dispose();
  }

  Future<void> _loadShader() async {
    try {
      _program = await ui.FragmentProgram.fromAsset(
        'shaders/${widget.shaderName}',
      );
      if (mounted) {
        setState(() {
          _programLoaded = true;
        });
      }
    } catch (e) {
      debugPrint('Failed to load shader: $e');
    }
  }

  Future<void> _updateTexture() async {
    final spectrum = widget.spectrum;
    final width = widget.spectrumWidth;
    final height = 1;

    final bytes = Uint8List(width * height * 4);
    for (int i = 0; i < width && i < spectrum.length; i++) {
      final value = (spectrum[i].clamp(0.0, 1.0) * 255).round().clamp(0, 255);
      final offset = i * 4;
      bytes[offset] = value; // R
      bytes[offset + 1] = value; // G
      bytes[offset + 2] = value; // B
      bytes[offset + 3] = 255; // A
    }

    final completer = Completer<ui.Image>();
    ui.decodeImageFromPixels(
      bytes,
      width,
      height,
      ui.PixelFormat.rgba8888,
      (image) => completer.complete(image),
    );

    final newTexture = await completer.future;
    if (mounted) {
      setState(() {
        _spectrumTexture?.dispose();
        _spectrumTexture = newTexture;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    // Wait for BOTH the shader AND the texture
    if (!_programLoaded || _spectrumTexture == null) {
      return Container(
        decoration: BoxDecoration(
          color: Colors.black,
          borderRadius: BorderRadius.circular(12),
        ),
        child: const Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              CircularProgressIndicator(color: Colors.cyanAccent),
              SizedBox(height: 8),
              Text(
                'Loading shader & texture...',
                style: TextStyle(color: Colors.white54),
              ),
            ],
          ),
        ),
      );
    }

    return ClipRRect(
      borderRadius: BorderRadius.circular(12),
      child: CustomPaint(
        // Now we know program and texture are 100% not null
        painter: _ShaderPainter(
          program: _program!,
          time: _frame * 0.016,
          frame: _frame,
          spectrumTexture: _spectrumTexture!, // Safe!
          uniforms: widget.uniforms,
        ),
        size: Size.infinite,
      ),
    );
  }
}

class _ShaderPainter extends CustomPainter {
  final ui.FragmentProgram program;
  final double time;
  final int frame;
  final ui.Image? spectrumTexture;
  final List<double> uniforms;

  _ShaderPainter({
    required this.program,
    required this.uniforms,
    required this.time,
    required this.frame,
    required this.spectrumTexture,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final shader = program.fragmentShader();

    // Set uniforms
    shader.setFloat(0, size.width); // iResolution.x
    shader.setFloat(1, size.height); // iResolution.y

    shader.setFloat(2, time); // iTime

    shader.setFloat(3, frame.toDouble()); // iFrame

    // iMouse - set to zero for automatic animation
    shader.setFloat(4, 0.0); // iMouse.x
    shader.setFloat(5, 0.0); // iMouse.y
    shader.setFloat(6, 0.0); // iMouse.z
    shader.setFloat(7, 0.0); // iMouse.w

    int index = 8;
    for (final v in uniforms) {
      shader.setFloat(index, v);
      index++;
    }

    // Pass spectrum texture as iChannel1 (sampler at index 0)
    if (spectrumTexture != null) {
      shader.setImageSampler(0, spectrumTexture!);
    }

    // Render
    canvas.drawRect(
      Rect.fromLTWH(0, 0, size.width, size.height),
      Paint()..shader = shader,
    );
  }

  @override
  bool shouldRepaint(_ShaderPainter oldDelegate) {
    return oldDelegate.time != time ||
        oldDelegate.frame != frame ||
        oldDelegate.spectrumTexture != spectrumTexture;
  }
}
