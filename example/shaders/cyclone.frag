#include <common/common_header.frag>

uniform sampler2D iChannel0;

float amp(float freq) {
    return texture(iChannel0, vec2(freq / 512.0)).x;
}

void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
    vec2 uv = fragCoord/iResolution.xy;
    float ratio = iResolution.y / iResolution.x;
	float c = 1.0;
    
    for (int i=0; i<100; i++) {
        float a = 6.0 * amp(float(i)) * amp(float(i)) * amp(float(i)) * amp(float(i)) + 2.0;
        vec2 center = 0.5 + 0.03 * vec2(sin(float(i)), cos(float(i))*ratio);
        vec2 vec = (uv - center) * vec2(1.0, ratio) * a * 1.1 *sin(float(i) / 40.0);
        c -= 0.19 * step(0.491, length(vec)) * step(0.5, 1.0-length(vec));
    }
    
	fragColor = vec4(vec3(0.35, 0.85713, 0.553123)+c,1.0);
}

#include <common/main_shadertoy.frag>