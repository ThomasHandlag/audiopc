// Code of animation for my music track:
// https://www.youtube.com/watch?v=iQ-OqHmJKmI
//
// Roman Meleshin - Passes
//---------------------------------------------
// http://romanmeleshin.art/
// 
// The code is distributed under the MIT license.
// Inspired by https://www.shadertoy.com/view/wtGfRy

#include <common/common_header.frag>

//#define PULSE
#define AA 1 // Antialias. 1 or more, 1 for fast.
#define MARCH_MAX_STEPS 100
#define BAR 3. // Beats per bar.
#define SEQ_BPM 210. // Beats per minute.

vec3 ro0;
vec3 ro;
#if defined(PULSE)
	float barPulse;
#endif

struct varsStruct{
	float iTime;
	float eA;
	float eB;
	float pA;
	float pD;
	float it;
	float s0;
	float pB;
	float pC;
	vec3 colBase;
	float colFactA;
	float colFactB;
	float fadeIn;
	float fadeOut;
	float fade;
	float clipTime;
};
varsStruct varsSeq[] = varsStruct[](
	varsStruct(
		/*iTime*/ 0., 
		/*eA*/ -2.5, 
		/*eB*/ 30., 
		/*pA*/ -1.5, 
		/*pD*/ -0.7, 
		/*it*/ 8., 
		/*s0*/ 2., 
		/*pB*/ 1., 
		/*pC*/ 1., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 23.75, 
		/*eA*/ -2.5, 
		/*eB*/ 30., 
		/*pA*/ -1.5, 
		/*pD*/ -0.7, 
		/*it*/ 8., 
		/*s0*/ 2., 
		/*pB*/ 1., 
		/*pC*/ 1., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 23.75, 
		/*eA*/ -2.5, 
		/*eB*/ 30., 
		/*pA*/ -1.5, 
		/*pD*/ -0.7, 
		/*it*/ 8., 
		/*s0*/ 2., 
		/*pB*/ 0., 
		/*pC*/ 0., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 47.75, 
		/*eA*/ -2.5, 
		/*eB*/ 30., 
		/*pA*/ -1.5, 
		/*pD*/ -0.7, 
		/*it*/ 8., 
		/*s0*/ 2., 
		/*pB*/ 0., 
		/*pC*/ 0., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 47.75, 
		/*eA*/ -3.8, 
		/*eB*/ 20., 
		/*pA*/ -1., 
		/*pD*/ -0.7, 
		/*it*/ 8., 
		/*s0*/ 2., 
		/*pB*/ 1., 
		/*pC*/ 1., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 71.75, 
		/*eA*/ -3.8, 
		/*eB*/ 20., 
		/*pA*/ -1., 
		/*pD*/ -0.7, 
		/*it*/ 8., 
		/*s0*/ 2., 
		/*pB*/ 1., 
		/*pC*/ 1., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 71.75, 
		/*eA*/ -2., 
		/*eB*/ 25., 
		/*pA*/ -1.9, 
		/*pD*/ -0.7, 
		/*it*/ 12., 
		/*s0*/ 2.5, 
		/*pB*/ 1., 
		/*pC*/ 1., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 95.75, 
		/*eA*/ -2., 
		/*eB*/ 25., 
		/*pA*/ -1.9, 
		/*pD*/ -0.7, 
		/*it*/ 12., 
		/*s0*/ 2.5, 
		/*pB*/ 1., 
		/*pC*/ 1., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 95.75, 
		/*eA*/ -2.5, 
		/*eB*/ 20., 
		/*pA*/ -1.5, 
		/*pD*/ -0.8, 
		/*it*/ 8., 
		/*s0*/ 2., 
		/*pB*/ 1., 
		/*pC*/ 1., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 119.75, 
		/*eA*/ -2.5, 
		/*eB*/ 20., 
		/*pA*/ -1.5, 
		/*pD*/ -0.8, 
		/*it*/ 8., 
		/*s0*/ 2., 
		/*pB*/ 1., 
		/*pC*/ 1., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 119.75, 
		/*eA*/ -2.5, 
		/*eB*/ 20., 
		/*pA*/ -1.5, 
		/*pD*/ -0.8, 
		/*it*/ 10., 
		/*s0*/ 2., 
		/*pB*/ 1., 
		/*pC*/ 1., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 143.75, 
		/*eA*/ -2.5, 
		/*eB*/ 20., 
		/*pA*/ -1.5, 
		/*pD*/ -0.8, 
		/*it*/ 10., 
		/*s0*/ 2., 
		/*pB*/ 1., 
		/*pC*/ 1., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 143.75, 
		/*eA*/ -2.5, 
		/*eB*/ 20., 
		/*pA*/ -1.5, 
		/*pD*/ -0.8, 
		/*it*/ 8., 
		/*s0*/ 2., 
		/*pB*/ 0., 
		/*pC*/ 0., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 167.75, 
		/*eA*/ -2.5, 
		/*eB*/ 20., 
		/*pA*/ -1.5, 
		/*pD*/ -0.8, 
		/*it*/ 8., 
		/*s0*/ 2., 
		/*pB*/ 0., 
		/*pC*/ 0., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 167.75, 
		/*eA*/ 1.9, 
		/*eB*/ 11., 
		/*pA*/ -1.7, 
		/*pD*/ 0.7, 
		/*it*/ 10., 
		/*s0*/ 2., 
		/*pB*/ 0., 
		/*pC*/ 0., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 191.75, 
		/*eA*/ 1.9, 
		/*eB*/ 11., 
		/*pA*/ -1.7, 
		/*pD*/ 0.7, 
		/*it*/ 10., 
		/*s0*/ 2., 
		/*pB*/ 0., 
		/*pC*/ 0., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 191.75, 
		/*eA*/ 1.9, 
		/*eB*/ 8., 
		/*pA*/ 1.7, 
		/*pD*/ 0.7, 
		/*it*/ 8., 
		/*s0*/ 3., 
		/*pB*/ 0., 
		/*pC*/ 0., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 215.75, 
		/*eA*/ 1.9, 
		/*eB*/ 8., 
		/*pA*/ 1.7, 
		/*pD*/ 0.7, 
		/*it*/ 8., 
		/*s0*/ 3., 
		/*pB*/ 0., 
		/*pC*/ 0., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 215.75, 
		/*eA*/ -2.5, 
		/*eB*/ 30., 
		/*pA*/ 1.5, 
		/*pD*/ -0.7, 
		/*it*/ 6., 
		/*s0*/ 3., 
		/*pB*/ 1., 
		/*pC*/ 1., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 233.75, 
		/*eA*/ -2.5, 
		/*eB*/ 30., 
		/*pA*/ 1.5, 
		/*pD*/ -0.7, 
		/*it*/ 6., 
		/*s0*/ 3., 
		/*pB*/ 1., 
		/*pC*/ 1., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 233.75, 
		/*eA*/ -2.4, 
		/*eB*/ 20., 
		/*pA*/ -1.5, 
		/*pD*/ -0.62, 
		/*it*/ 6., 
		/*s0*/ 3., 
		/*pB*/ 1., 
		/*pC*/ 1., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 251.75, 
		/*eA*/ -2.4, 
		/*eB*/ 20., 
		/*pA*/ -1.5, 
		/*pD*/ -0.62, 
		/*it*/ 6., 
		/*s0*/ 3., 
		/*pB*/ 1., 
		/*pC*/ 1., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 251.75, 
		/*eA*/ -2.3, 
		/*eB*/ 30., 
		/*pA*/ -1.5, 
		/*pD*/ -0.7, 
		/*it*/ 8., 
		/*s0*/ 2., 
		/*pB*/ 0., 
		/*pC*/ 0., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 275.75, 
		/*eA*/ -2.3, 
		/*eB*/ 30., 
		/*pA*/ -1.5, 
		/*pD*/ -0.7, 
		/*it*/ 8., 
		/*s0*/ 2., 
		/*pB*/ 0., 
		/*pC*/ 0., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 275.75, 
		/*eA*/ -2.5, 
		/*eB*/ 30., 
		/*pA*/ -1.5, 
		/*pD*/ -0.7, 
		/*it*/ 10., 
		/*s0*/ 2., 
		/*pB*/ 0., 
		/*pC*/ 0., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 299.75, 
		/*eA*/ -2.5, 
		/*eB*/ 30., 
		/*pA*/ -1.5, 
		/*pD*/ -0.7, 
		/*it*/ 10., 
		/*s0*/ 2., 
		/*pB*/ 0., 
		/*pC*/ 0., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 299.75, 
		/*eA*/ -3.8, 
		/*eB*/ 20., 
		/*pA*/ -1., 
		/*pD*/ -0.7, 
		/*it*/ 10., 
		/*s0*/ 2., 
		/*pB*/ 1., 
		/*pC*/ 1., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 323.75, 
		/*eA*/ -3.8, 
		/*eB*/ 20., 
		/*pA*/ -1., 
		/*pD*/ -0.7, 
		/*it*/ 10., 
		/*s0*/ 2., 
		/*pB*/ 1., 
		/*pC*/ 1., 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 323.75, 
		/*eA*/ -2.5, 
		/*eB*/ 30., 
		/*pA*/ -1.5, 
		/*pD*/ -0.7, 
		/*it*/ 8., 
		/*s0*/ 2., 
		/*pB*/ 0.25, 
		/*pC*/ 0.25, 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 347.75, 
		/*eA*/ -2.5, 
		/*eB*/ 30., 
		/*pA*/ -1.5, 
		/*pD*/ -0.7, 
		/*it*/ 8., 
		/*s0*/ 2., 
		/*pB*/ 0.25, 
		/*pC*/ 0.25, 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 347.75, 
		/*eA*/ -2.5, 
		/*eB*/ 30., 
		/*pA*/ -1.5, 
		/*pD*/ -0.7, 
		/*it*/ 6., 
		/*s0*/ 2., 
		/*pB*/ 0.25, 
		/*pC*/ 0.25, 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 365.75, 
		/*eA*/ -2.5, 
		/*eB*/ 30., 
		/*pA*/ -1.5, 
		/*pD*/ -0.7, 
		/*it*/ 6., 
		/*s0*/ 2., 
		/*pB*/ 0.25, 
		/*pC*/ 0.25, 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 365.75, 
		/*eA*/ -2.5, 
		/*eB*/ 30., 
		/*pA*/ -1.5, 
		/*pD*/ -0.7, 
		/*it*/ 6., 
		/*s0*/ 1., 
		/*pB*/ 0.25, 
		/*pC*/ 0.25, 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 389.75, 
		/*eA*/ -2.5, 
		/*eB*/ 30., 
		/*pA*/ -1.5, 
		/*pD*/ -0.7, 
		/*it*/ 6., 
		/*s0*/ 1., 
		/*pB*/ 0.25, 
		/*pC*/ 0.25, 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 389.75, 
		/*eA*/ -2.5, 
		/*eB*/ 30., 
		/*pA*/ -1.5, 
		/*pD*/ -0.7, 
		/*it*/ 8., 
		/*s0*/ 1., 
		/*pB*/ 0.25, 
		/*pC*/ 0.25, 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 413.75, 
		/*eA*/ -2.5, 
		/*eB*/ 30., 
		/*pA*/ -1.5, 
		/*pD*/ -0.7, 
		/*it*/ 8., 
		/*s0*/ 1., 
		/*pB*/ 0.25, 
		/*pC*/ 0.25, 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 413.75, 
		/*eA*/ -2.5, 
		/*eB*/ 20., 
		/*pA*/ -2., 
		/*pD*/ -0.7, 
		/*it*/ 8., 
		/*s0*/ 1., 
		/*pB*/ 0.25, 
		/*pC*/ 0.25, 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 437.75, 
		/*eA*/ -2.5, 
		/*eB*/ 20., 
		/*pA*/ -2., 
		/*pD*/ -0.7, 
		/*it*/ 8., 
		/*s0*/ 1., 
		/*pB*/ 0.25, 
		/*pC*/ 0.25, 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 437.75, 
		/*eA*/ -2.4, 
		/*eB*/ 20., 
		/*pA*/ -2., 
		/*pD*/ -0.7, 
		/*it*/ 8., 
		/*s0*/ 1., 
		/*pB*/ 0.25, 
		/*pC*/ 0.25, 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 461.75, 
		/*eA*/ -2.4, 
		/*eB*/ 20., 
		/*pA*/ -2., 
		/*pD*/ -0.7, 
		/*it*/ 8., 
		/*s0*/ 1., 
		/*pB*/ 0.25, 
		/*pC*/ 0.25, 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 461.75, 
		/*eA*/ -2.4, 
		/*eB*/ 30., 
		/*pA*/ -2., 
		/*pD*/ -0.7, 
		/*it*/ 8., 
		/*s0*/ 1., 
		/*pB*/ 0.25, 
		/*pC*/ 0.25, 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 485.75, 
		/*eA*/ -2.4, 
		/*eB*/ 30., 
		/*pA*/ -2., 
		/*pD*/ -0.7, 
		/*it*/ 8., 
		/*s0*/ 1., 
		/*pB*/ 0.25, 
		/*pC*/ 0.25, 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 6., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 485.75, 
		/*eA*/ -2.4, 
		/*eB*/ 30., 
		/*pA*/ -2., 
		/*pD*/ -0.6, 
		/*it*/ 8., 
		/*s0*/ 1., 
		/*pB*/ 0.25, 
		/*pC*/ 0.25, 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 24., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	),
	varsStruct(
		/*iTime*/ 528., 
		/*eA*/ -2.4, 
		/*eB*/ 30., 
		/*pA*/ -2., 
		/*pD*/ -0.6, 
		/*it*/ 8., 
		/*s0*/ 1., 
		/*pB*/ 0.25, 
		/*pC*/ 0.25, 
		/*colBase*/ vec3(0.2,0.9,0.3), 
		/*colFactA*/ 3., 
		/*colFactB*/ 0.00005, 
		/*fadeIn*/ 3., 
		/*fadeOut*/ 24., 
		/*fade*/ 1., 
		/*clipTime*/ 0.
	)
);
varsStruct vars;
void varsInit(){
	const float beatTime = 60. / SEQ_BPM;
	float loopLength = varsSeq[varsSeq.length()-1].iTime * beatTime;
	float loopT = mod(iTime, loopLength);
	float clipT;
	float clipL;
	varsStruct a;
	varsStruct b;
	float t;
	for(int i = 0; i < varsSeq.length()-1; i++){
		float sTimeNext = varsSeq[i+1].iTime * beatTime;
		if(sTimeNext > loopT){
			float sTime = varsSeq[i].iTime * beatTime;
			a = varsSeq[i];
			b = varsSeq[i+1];
			clipT = loopT - sTime;
			clipL = sTimeNext - sTime;
			t = clipT/clipL;
			break;
		}
	}
	vars.iTime = loopT;
	vars.clipTime = t;
	vars.eA = mix(a.eA, b.eA, t);
	vars.eB = mix(a.eB, b.eB, t);
	vars.pA = mix(a.pA, b.pA, t);
	vars.pD = mix(a.pD, b.pD, t);
	vars.it = mix(a.it, b.it, t);
	vars.s0 = mix(a.s0, b.s0, t);
	vars.pB = mix(a.pB, b.pB, t);
	vars.pC = mix(a.pC, b.pC, t);
	vars.colBase = mix(a.colBase, b.colBase, t);
	vars.colFactA = mix(a.colFactA, b.colFactA, t);
	vars.colFactB = mix(a.colFactB, b.colFactB, t);
	vars.fade = 1.;
	if(a.fadeIn != 0.){
		vars.fade *= clamp(clipT / (a.fadeIn * beatTime), 0., 1.);
	}
	if(a.fadeOut != 0.){
		vars.fade *= clamp((clipL - clipT) / (a.fadeOut * beatTime), 0., 1.);
	}
	vars.clipTime = mix(a.clipTime, b.clipTime, t);
}

mat3 rotMat(vec3 v, float c, float s){
	float k = (1. - c);
	return mat3(
		k * v.x * v.x + c,			k * v.x * v.y - s * v.z,	k * v.x * v.z + s * v.y,
		k * v.x * v.y + s * v.z,	k * v.y * v.y +c,			k * v.y * v.z - s * v.x,
		k * v.x * v.z - s * v.y,	k * v.y * v.z + s * v.x,	k * v.z * v.z + c
	);
}
mat3 rotMat(vec3 v, float angle){
	return rotMat(v, cos(angle), sin(angle));
}
mat3 rotMat(vec3 z, vec3 d){
	vec3 cr = cross(z, d);
	return rotMat(cr, dot(z, d), length(cr));
}
vec3 opRepeat(vec3 p, float repW){
	float repWD = repW / 2.;
	return mod(p + repWD, repW) - repWD;
}

void tune(){ 
    if(vars.eA > 0.){
        vars.pB = abs(vars.pB);
    }else if(vars.eA < 0.){
        vars.it = ceil(vars.it/2.)*2.;
    }
    #if defined(PULSE)
    	vars.s0 *= 1.-0.15*barPulse;
    	vars.pA *= 1.+0.001*barPulse;
    #endif
}
vec4 deFractal(vec3 p){
	float s = vars.s0;
	p = abs(p);
    p += vars.pA;
    for(float i=0.; i<vars.it; i++){
        p -= vars.pB;
        p = abs(p);
		p = vars.pC - p;
		float e = vars.eA/min(dot(p,p),vars.eB);
		s*=e;
		p=p*e + vars.pD;
	}
	float dist = abs(p.z)/s;
	dist += .001;
	vec3 col = vars.colBase * 10. + log(s * vars.colFactA);
	col = vars.colFactB * abs(cos(col))/dot(p,p)/dist;
	return vec4(col, dist);
}

vec4 map(vec3 p){
	p -= ro0;
	p = opRepeat(p, 2.);
	return deFractal(p);
}
vec3 march(vec3 ro, vec3 rd){
	vec4 res = vec4(0.);
	for(int i=0; i<MARCH_MAX_STEPS; i++){
		vec3 p = ro + res.a * rd;
		vec4 m = map(p);
		res += m;
	 }
	 return res.rgb;
}

void mainImage( out vec4 fragColor, in vec2 fragCoord ){
	varsInit();
	
	const float beatTime = 60./SEQ_BPM;
	const float barTime = BAR*beatTime;
	#if defined(PULSE)
		float iTimeInBars = vars.iTime / barTime;
		barPulse = pow(1. - abs(iTimeInBars - round(iTimeInBars))*2., 3.);
	#endif
	
	tune();

	const float foc = 1.;
	ro0 = vec3(0, 0, 0);
	ro = ro0 + vec3(0,0,iTime/barTime/4.);
	vec3 col =vec3(0.);
#if AA>1
	for(int i = 0; i < AA; ++i){
		for(int j = 0; j < AA; ++j){
			vec2 ao = vec2(float(i),float(j))/float(AA) - .5;
			vec2 uv = (fragCoord.xy + ao - .5*iResolution.xy)/iResolution.y;
#else
			vec2 uv = (fragCoord.xy - .5*iResolution.xy)/iResolution.y;
#endif
			vec3 rd = normalize(vec3(uv.x, uv.y, foc));
			col += march(ro, rd);
			
#if AA>1
		}
	}
	col /= float(AA*AA);
#endif
	fragColor = vec4(col * vars.fade, 1);
}

#include <common/main_shadertoy.frag>