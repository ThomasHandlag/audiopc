// This include is mandatory for all shaders since the [LayerBuffer] always
// set the uniforms defined here
#version 460 core
#include <flutter/runtime_effect.glsl>
precision mediump float;

uniform vec2 iResolution;   // viewport resolution (in pixels)
uniform float iTime;        // shader playback time (in seconds)
uniform float iFrame;       // shader playback frame
uniform vec4 iMouse;        // mouse pixel coords. xy: current (if MLB down), zw: click

out vec4 fragColor;
