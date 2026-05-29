uniform float iTime;

#define A(x, y) abs(dot(sin(x), vec3(y)))
void mainImage(out vec4 o, vec2 u) {
    vec3 iResolution = vec3(1920, 1080, 0);
    
    float i, d, s, l, t = iTime+sin(iTime) / 2.;
    vec3 q, p, r = iResolution;
    for(o *= i; i++ < 1e2;
        l = length(vec2(p.x, d - 130.)),
        p *= vec3(.125, .6, 1),
        d += s = min(.2 + .4*abs(q.y + 2e1 + sin(l*.2 - t*1e1)),
                     .3 + .3*abs(3. - length(p.xy)) - min(0., q.y + 12.)),
        o += 1./s)
        for(p = vec3((u + u - r.xy)/r.y*d, d - 7e1),
            q = p,
            p.yz *= mat2(cos(1.2 + vec4(0, 33, 11, 0))),
            p.z += t*3e1,
            s = .03; s < 4.; s += s)
            p.yz -= A(t + t + .32*p/s, s),
            q += A(.3*q.z + t + .7*q/s, s/8.);

    o = tanh(o*o /  2e4);
    
}

void main() {
    mainImage(gl_FragColor, gl_FragCoord.xy);
}