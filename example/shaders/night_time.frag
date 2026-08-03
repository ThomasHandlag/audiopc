#include <common/common_header.frag>

uniform sampler2D iChannel0;

#define AA 16.

#define FOCAL_LENGTH 0.6

#define VISUALIZER_BARS 20.
#define VISUALIZER_GAP 0.75

//#define iTime 6.52

mat2 rotMat(float r){ return  mat2(cos(r), -sin(r), sin(r), cos(r)); }

vec3 rotX(in vec3 p, float r)
{
    p.yz *= rotMat(r); return p;
}

vec3 rotY(in vec3 p, float r)
{
    p.xz *= rotMat(r); return p;
}
 
vec3 rotZ(in vec3 p, float r)
{
    p.xy *= rotMat(r); return p;
}

float hash11(float p)
{
    p = fract(p * .1031);
    p *= p + 33.33;
    p *= p + p;
    return fract(p);
}

vec2 hash12(float p)
{
    vec3 p3 = fract(vec3(p,p,p) * vec3(.1031, .1030, .0973));
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.xx+p3.yz)*p3.zy);
}

vec2 hash22(vec2 p)
{
    vec3 p3 = fract(vec3(p.xyx) * vec3(.1031, .1030, .0973));
    p3 += dot(p3, p3.yzx+33.33);
    return fract((p3.xx+p3.yz)*p3.zy);
}

float hash21(vec2 p)
{
    vec3 p3 = fract(vec3(p.xyx) * .1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

float cosmooth(float x) { return 1.-cos(3.141593*x)*0.5-0.5; }

float shash(float x)
{
    float fx = floor(x);
    float s = mod(fx, 2.)*2.-1.;
    float a = hash11(fx)*s;
    float b = hash11(fx+1.)*-s;
    return mix(a, b, cosmooth(fract(x)))*0.5;
}

float pingPong(float x)
{
    return abs(mod(x, 2.)-1.);
}

vec3 planeIntersect(in vec3 ro, vec3 rd, float y)
{
    ro.xz = ro.xz+rd.xz*(y-ro.y)/rd.y;
    return vec3(ro.x, y, ro.z);
}

vec3 getEnvironmentMap(vec3 rd, vec3 l)
{
    float ld = dot(rd, l);
    vec3 mainColor = (1./(max(1e-10, 1.-ld+0.003))*0.01
                     +0.02/(abs(ld-0.95)+0.07) // ring
                     )
                     *vec3(0.733,0.302,0.769) // coloring
                     +exp(-rd.y*3.)*vec3(0.125,0.075,0.110) // sky gradient
                     ;
    vec2 visUv = planeIntersect(vec3(0), rd.zxy, -1.).xy+0.25/VISUALIZER_BARS;
    float vis = step(rd.y, pow(texture(iChannel0, vec2(1.-pingPong(round((1.-abs(visUv).x)*VISUALIZER_BARS)/VISUALIZER_BARS), 0)).r, 4.)*0.3);
    vis *= step(VISUALIZER_GAP, VISUALIZER_BARS*abs((1.-pingPong(round((1.-abs(visUv+0.05).x)*VISUALIZER_BARS)/VISUALIZER_BARS))-(1.-pingPong(1.-abs(visUv+0.5/VISUALIZER_BARS).x))));
    return max(vec3(0), mainColor + vis*0.3*smoothstep(-0.05, 0.2, rd.y));
    //return vec3(abs(visUv).xx, 0);
    //return vec3(vec2(step(0.01, abs((1.-pingPong(round((1.-abs(visUv).x)*10.)/10.))-(1.-pingPong(1.-abs(visUv).x))))), 0);
}

float getHeight(vec2 p)
{
    float r = 0.;
    for(float i = 0.; i < 3.; i++)
    {
        float e = exp2(-i);
        r += abs(sin(p.x/e+iTime))*e;
        p = rotZ(p.xyx, hash11(i*25.)*0.2).xy;
    }
    return r;
}

vec2 heightNormal(vec2 p, float dist)
{
    vec2 e = vec2(0.2, 0);
    return normalize(vec2(getHeight(p+e.xy)-getHeight(p-e.xy),
                          getHeight(p+e.yx)-getHeight(p-e.yx)));
}

vec3 gradToVec(vec2 d)
{
    vec3 a = normalize(vec3(1, -d.x, 0)),
         b = normalize(vec3(0, -d.y, 1));
    return cross(a, b);
}

vec3 halfVec(vec3 a, vec3 b)
{
    return normalize(a+b);
}

void _mainImage( out vec4 fragColor, in vec2 fragCoord )
{
    vec2 nuv = fragCoord/iResolution.xy;
    vec2 uv = (fragCoord-iResolution.xy*0.5)/iResolution.y;
    vec2 m =  (iMouse.xy-iResolution.xy*0.5)/iResolution.y*step(0.01, iMouse.z);
    
    vec2 rot = vec2(3.14*-.5, 0)-m.xy*4.-vec2(shash(iTime*0.25), shash(iTime*0.25+0.7071067812))*0.1;
    vec3 ro = vec3(0, 1, 0);
    vec3 rd = rotY(rotX(normalize(vec3(uv, FOCAL_LENGTH)), rot.y), rot.x);
    vec3 center = rotY(rotX(normalize(vec3(0, 0, FOCAL_LENGTH)), rot.y), rot.x);;
    vec3 lgt = rotZ(vec3(1, 0, 0), 0.2);
    
    vec3 pli = planeIntersect(ro, rd, 0.);
    float blend = smoothstep(-0.05, 0., rd.y);
    
    vec3 planeNormal = gradToVec(0.2*heightNormal(clamp(pli.xz, vec2(-1e4), vec2(1e4)), 1.)/max(1., distance(ro, pli)*0.5));
    
    vec3 env = getEnvironmentMap(mix(reflect(rd, planeNormal), rd, blend), lgt);
    float vignette = 4.*nuv.x*(1.-nuv.x)*4.*nuv.y*(1.-nuv.y)*0.6+0.4;
    float glow = pow(max(0., dot(rd, halfVec(center, lgt))), 12.)*0.3*mix(0.2, 1.5, pow(max(0., dot(center, lgt)), 25.));

    fragColor = vec4(sqrt((env+glow)*vignette), 1.0);
    //fragColor = vec4(getEnvironmentMap(rd, lgt), 0);
}

vec4 stochasticAA(vec2 fc, vec4 m)
{
    vec4 res = vec4(0);
    float st = 1.0/(AA-1.);
    vec4 color;
    _mainImage(color, fc);
    res += color;
    if(AA > 1.)
    {
        for(float x = 0.; x < 1.; x += st)
        {
            vec4 color;
            _mainImage(color, fc + (hash22((x+iTime+fc)*25.)-0.5));
            res += color;
        }
    }
    return res/AA;
}

vec3 dither(vec3 color, vec2 coord, float steps)
{
    vec3 reduce = floor(color*steps)/steps;
    vec3 error = color-reduce;
    return reduce+step(vec3(hash21(coord)), error*steps)/steps;
}

void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
    fragColor = vec4(dither(stochasticAA(fragCoord, iMouse).rgb, fragCoord, 256.), 1);
}

#include <common/main_shadertoy.frag>