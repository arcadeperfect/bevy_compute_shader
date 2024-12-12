#define_import_path compute::anoise
// Modulo 289 without a division
fn mod289(x: f32) -> f32 {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn mod289_v3(x: vec3<f32>) -> vec3<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

// Permutation polynomial (ring size 289 = 17*17)
fn permute(x: vec3<f32>) -> vec3<f32> {
    return mod289_v3(((x * 34.0) + 10.0) * x);
}

// Hashed 2-D gradients with an extra rotation.
fn rgrad2(p: vec2<f32>, rot: f32) -> vec2<f32> {
    // First permute p.x, then add p.y and permute again
    let px_permuted = permute(vec3<f32>(p.x, 0.0, 0.0));
    let sum_permuted = permute(vec3<f32>(px_permuted.x + p.y, 0.0, 0.0));
    let u = sum_permuted.x * 0.0243902439 + rot; // Rotate by shift
    let u1 = fract(u) * 6.28318530718; // 2*pi
    return vec2<f32>(cos(u1), sin(u1));
}

fn mod_f32(x: f32, y: f32) -> f32 {
    return x - y * floor(x/y);
}

fn mod_v3(x: vec3<f32>, y: f32) -> vec3<f32> {
    return x - y * floor(x/y);
}

// 2-D tiling simplex noise with rotating gradients and analytical derivative.
// The first component of the 3-element return vector is the noise value,
// and the second and third components are the x and y partial derivatives.
fn psrdnoise2(pos: vec2<f32>, per: vec2<f32>, rot: f32) -> vec3<f32> {
    // Hack: offset y slightly to hide some rare artifacts
    let pos1 = vec2<f32>(pos.x, pos.y + 0.001);
    
    // Skew to hexagonal grid
    let uv = vec2<f32>(pos1.x + pos1.y*0.5, pos1.y);
    
    let i0 = floor(uv);
    let f0 = fract(uv);
    
    // Traversal order
    let i1 = select(vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), f0.x > f0.y);

    // Unskewed grid points in (x,y) space
    let p0 = vec2<f32>(i0.x - i0.y * 0.5, i0.y);
    let p1 = vec2<f32>(p0.x + i1.x - i1.y * 0.5, p0.y + i1.y);
    let p2 = vec2<f32>(p0.x + 0.5, p0.y + 1.0);

    // Integer grid point indices in (u,v) space
    let i2 = i0 + vec2<f32>(1.0, 1.0);

    // Vectors in unskewed (x,y) coordinates from
    // each of the simplex corners to the evaluation point
    let d0 = pos1 - p0;
    let d1 = pos1 - p1;
    let d2 = pos1 - p2;

    // Wrap i0, i1 and i2 to the desired period before gradient hashing:
    // wrap points in (x,y), map to (u,v)
    let xw = mod_v3(vec3<f32>(p0.x, p1.x, p2.x), per.x);
    let yw = mod_v3(vec3<f32>(p0.y, p1.y, p2.y), per.y);
    let iuw = xw + 0.5 * yw;
    let ivw = yw;
    
    // Create gradients from indices
    let g0 = rgrad2(vec2<f32>(iuw.x, ivw.x), rot);
    let g1 = rgrad2(vec2<f32>(iuw.y, ivw.y), rot);
    let g2 = rgrad2(vec2<f32>(iuw.z, ivw.z), rot);

    // Gradients dot vectors to corners
    let w = vec3<f32>(
        dot(g0, d0),
        dot(g1, d1),
        dot(g2, d2)
    );
    
    // Radial weights from corners
    // 0.8 is the square of 2/sqrt(5), the distance from
    // a grid point to the nearest simplex boundary
    let t = vec3<f32>(
        0.8 - dot(d0, d0),
        0.8 - dot(d1, d1),
        0.8 - dot(d2, d2)
    );

    // Partial derivatives for analytical gradient computation
    let dtdx = -2.0 * vec3<f32>(d0.x, d1.x, d2.x);
    let dtdy = -2.0 * vec3<f32>(d0.y, d1.y, d2.y);

    // Set influence of each surflet to zero outside radius sqrt(0.8)
    let t1 = max(t, vec3<f32>(0.0));

    // Fourth power of t (and third power for derivative)
    let t2 = t1 * t1;
    let t4 = t2 * t2;
    let t3 = t2 * t1;
    
    // Final noise value is:
    // sum of ((radial weights) times (gradient dot vector from corner))
    let n = dot(t4, w);
    
    // Final analytical derivative (gradient of a sum of scalar products)
    let dt0 = vec2<f32>(dtdx.x, dtdy.x) * 4.0 * t3.x;
    let dn0 = t4.x * g0 + dt0 * w.x;
    let dt1 = vec2<f32>(dtdx.y, dtdy.y) * 4.0 * t3.y;
    let dn1 = t4.y * g1 + dt1 * w.y;
    let dt2 = vec2<f32>(dtdx.z, dtdy.z) * 4.0 * t3.z;
    let dn2 = t4.z * g2 + dt2 * w.z;

    // Return vec3 with noise value and derivatives
    return 11.0 * vec3<f32>(n, (dn0 + dn1 + dn2).x, (dn0 + dn1 + dn2).y);
}