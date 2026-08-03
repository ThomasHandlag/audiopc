// Fork of "Color Ripples [273]" by diatribes. https://shadertoy.com/view/33jSDc
// 2025-07-24 12:02:32


// INTO: ---> https://www.shadertoy.com/view/lXcfRf <--- that 

#include <common/common_header.frag>

uniform sampler2D iChannel0;

void mainImage(out vec4 o, vec2 u) {
    float i,d,s;
    for(o*=i; i++<1e2; ) {
        vec3 p = d * normalize(vec3(u+u,0) - iResolution.xyx );
        for (s = .1; s < 1.;
            p -= dot(sin(p * s * 16.), vec3(.01)) / s,
            p.xz *= mat2(cos(.3*iTime+vec4(0,33,11,0))),
            s += s);
        d += s = .01 + abs(p.y);
        o += (1.+cos(d+vec4(4,2,1,0))) / s;
    }
    o = tanh(o / 6e3);
    o *= texture(iChannel0, u / iResolution.xy);
}

#include <common/main_shadertoy.frag>