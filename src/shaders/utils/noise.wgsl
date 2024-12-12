#define_import_path compute::noise


// Noise functions from https://gist.github.com/munrocket/236ed5ba7e409b8bdf1ff6eca5dcdc39





//  MIT License. © Ian McEwan, Stefan Gustavson, Munrocket, Johan Helsing
//
fn mod289(x: vec2f) -> vec2f {
    return x - floor(x * (1. / 289.)) * 289.;
}

fn mod289_3(x: vec3f) -> vec3f {
    return x - floor(x * (1. / 289.)) * 289.;
}

fn permute3(x: vec3f) -> vec3f {
    return mod289_3(((x * 34.) + 1.) * x);
}


//  MIT License. © Ian McEwan, Stefan Gustavson, Munrocket
fn noise2(v: vec2f) -> f32 {
    let C = vec4(
        0.211324865405187, // (3.0-sqrt(3.0))/6.0
        0.366025403784439, // 0.5*(sqrt(3.0)-1.0)
        -0.577350269189626, // -1.0 + 2.0 * C.x
        0.024390243902439 // 1.0 / 41.0
    );

    // First corner
    var i = floor(v + dot(v, C.yy));
    let x0 = v - i + dot(i, C.xx);

    // Other corners
    var i1 = select(vec2(0., 1.), vec2(1., 0.), x0.x > x0.y);

    // x0 = x0 - 0.0 + 0.0 * C.xx ;
    // x1 = x0 - i1 + 1.0 * C.xx ;
    // x2 = x0 - 1.0 + 2.0 * C.xx ;
    var x12 = x0.xyxy + C.xxzz;
    x12.x = x12.x - i1.x;
    x12.y = x12.y - i1.y;

    // Permutations
    i = mod289(i); // Avoid truncation effects in permutation

    var p = permute3(permute3(i.y + vec3(0., i1.y, 1.)) + i.x + vec3(0., i1.x, 1.));
    var m = max(0.5 - vec3(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw)), vec3(0.));
    m *= m;
    m *= m;

    // Gradients: 41 points uniformly over a line, mapped onto a diamond.
    // The ring size 17*17 = 289 is close to a multiple of 41 (41*7 = 287)
    let x = 2. * fract(p * C.www) - 1.;
    let h = abs(x) - 0.5;
    let ox = floor(x + 0.5);
    let a0 = x - ox;

    // Normalize gradients implicitly by scaling m
    // Approximation of: m *= inversesqrt( a0*a0 + h*h );
    m *= 1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h);

    // Compute final noise value at P
    let g = vec3(a0.x * x0.x + h.x * x0.y, a0.yz * x12.xz + h.yz * x12.yw);
    return 130. * dot(m, g);
}

// fn fbm(p1: vec2f) -> f32 {
//     var m2: mat2x2f = mat2x2f(vec2f(0.8, 0.6), vec2f(-0.6, 0.8));
//     var f: f32 = 0.;
//     var p = p1;
//     f = f + 0.5000 * noise2(p); p = m2 * p * 2.02;
//     f = f + 0.2500 * noise2(p); p = m2 * p * 2.03;
//     f = f + 0.1250 * noise2(p); p = m2 * p * 2.01;
//     f = f + 0.0625 * noise2(p);
//     return f / 0.9375;
// }

// fmb with fixed lacunarity
fn fbm(p1: vec2f) -> f32 {
    var m2: mat2x2f = mat2x2f(vec2f(0.8, 0.6), vec2f(-0.6, 0.8));
    var f: f32 = 0.;
    var p = p1;
    f = f + 0.5000 * noise2(p); p = m2 * p * 2.02;
    f = f + 0.2500 * noise2(p); p = m2 * p * 2.03;
    f = f + 0.1250 * noise2(p); p = m2 * p * 2.01;
    f = f + 0.0625 * noise2(p); p = m2 * p * 2.02;
    f = f + 0.0313 * noise2(p); p = m2 * p * 2.01;
    f = f + 0.0156 * noise2(p);
    return f / 0.9844; // Updated normalization factor
}

// fmb with dynamic lacunarity
fn fbml(p1: vec2f, l: f32) -> f32 {
    var m2: mat2x2f = mat2x2f(vec2f(0.8, 0.6), vec2f(-0.6, 0.8));
    var f: f32 = 0.;
    var p = p1;
    f = f + 0.5000 * noise2(p); p = m2 * p * 2.02 * l;
    f = f + 0.2500 * noise2(p); p = m2 * p * 2.03 * l;
    f = f + 0.1250 * noise2(p); p = m2 * p * 2.01 * l;
    f = f + 0.0625 * noise2(p); p = m2 * p * 2.02 * l;
    f = f + 0.0313 * noise2(p); p = m2 * p * 2.01 * l;
    f = f + 0.0156 * noise2(p);
    return f / 0.9844; // Updated normalization factor
}

// Good and fast integer hash
// https://www.pcg-random.org/
fn pcg(n: u32) -> u32 {
    var h = n * 747796405u + 2891336453u;
    h = ((h >> ((h >> 28u) + 4u)) ^ h) * 277803737u;
    return (h >> 22u) ^ h;
}

fn pcg2d(p: vec2u) -> vec2u {
    var v = p * 1664525u + 1013904223u;
    v.x += v.y * 1664525u; v.y += v.x * 1664525u;
    v ^= v >> vec2u(16u);
    v.x += v.y * 1664525u; v.y += v.x * 1664525u;
    v ^= v >> vec2u(16u);
    return v;
}

// http://www.jcgt.org/published/0009/03/02/
fn pcg3d(p: vec3u) -> vec3u {
    var v = p * 1664525u + 1013904223u;
    v.x += v.y*v.z; v.y += v.z*v.x; v.z += v.x*v.y;
    v ^= v >> vec3u(16u);
    v.x += v.y*v.z; v.y += v.z*v.x; v.z += v.x*v.y;
    return v;
}

// http://www.jcgt.org/published/0009/03/02/
fn pcg4d(p: vec4u) -> vec4u {
    var v = p * 1664525u + 1013904223u;
    v.x += v.y*v.w; v.y += v.z*v.x; v.z += v.x*v.y; v.w += v.y*v.z;
    v ^= v >> vec4u(16u);
    v.x += v.y*v.w; v.y += v.z*v.x; v.z += v.x*v.y; v.w += v.y*v.z;
    return v;
}

// Hash based rand

fn rand11(f: f32) -> f32 { return f32(pcg(bitcast<u32>(f))) / f32(0xffffffff); }
fn rand22(f: vec2f) -> vec2f { return vec2f(pcg2d(bitcast<vec2u>(f))) / f32(0xffffffff); }
fn rand33(f: vec3f) -> vec3f { return vec3f(pcg3d(bitcast<vec3u>(f))) / f32(0xffffffff); }
fn rand44(f: vec4f) -> vec4f { return vec4f(pcg4d(bitcast<vec4u>(f))) / f32(0xffffffff); }