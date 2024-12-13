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


fn fbma(
    pos: vec2<f32>,
    octaves: u32,
    lacunarity: f32,
    gain: f32,
    base_period: f32,
    rot: f32,
) -> vec3<f32> {
    var value = 0.0;
    var dx = 0.0;
    var dy = 0.0;

    var amplitude = 1.0;
    var frequency = 1.0;

    for (var i = 0u; i < octaves; i = i + 1u) {
        let per = vec2<f32>(base_period * frequency, base_period * frequency);
        let result = psrdnoise2(pos * frequency, per, rot);
        // result.x = noise value
        // result.y = derivative wrt x_scaled
        // result.z = derivative wrt y_scaled

        // Scale the derivatives back to original space:
        // d/dx_original = d/dx_scaled * frequency
        // d/dy_original = d/dy_scaled * frequency

        value = value + result.x * amplitude;
        dx = dx + (result.y * amplitude * frequency);
        dy = dy + (result.z * amplitude * frequency);

        amplitude = amplitude * gain;
        frequency = frequency * lacunarity;
    }

    return vec3<f32>(value, dx, dy);
}

fn terrain_gpt(
    initial_pos: vec2<f32>,
    octaves: u32,
    lacunarity: f32,
    gain: f32,
    base_period: f32,
    rot: f32
) -> f32 {
    var a = 0.0;   // Accumulated noise value
    var b = 1.0;   // Amplitude for this octave
    var d = vec2<f32>(0.0, 0.0); // Accumulated derivatives
    var p = initial_pos;

    // A rotation matrix to alter the pattern each octave, as in Quílez's code
    let m = mat2x2<f32>(
        0.8, -0.6,
        0.6,  0.8
    );
    
    var frequency = 1.0;

    for (var i = 0u; i < octaves; i = i + 1u) {
        let per = vec2<f32>(base_period * frequency, base_period * frequency);
        
        // psrdnoise2 returns vec3: (noise_value, dx, dy)
        let n = psrdnoise2(p, per, rot);
        
        // Extract derivatives
        let dx = n.y;
        let dy = n.z;
        
        // Accumulate derivatives
        d = d + vec2<f32>(dx, dy);
        
        // Incorporate derivatives into the fbm accumulation
        // This formula introduces slope-dependent variation
        a = a + b * n.x / (1.0 + dot(d, d));

        // Prepare for next octave
        b = b * gain;
        frequency = frequency * lacunarity;
        p = m * p * 2.0; // Rotating and scaling coordinates each octave
    }

    return a;
}

fn terrain_claude(
    pos: vec2<f32>,
    octaves: u32,
    lacunarity: f32,
    gain: f32,
    base_period: f32,
    rot: f32,
) -> f32 {
    // Initialize accumulation values
    var value = 0.0;
    var amplitude = 1.0;
    var frequency = 1.0;
    
    // Track derivatives for gradient contribution
    var d = vec2<f32>(0.0, 0.0);

    for (var i = 0u; i < octaves; i = i + 1u) {
        let per = vec2<f32>(base_period * frequency, base_period * frequency);
        
        // Get noise value and derivatives
        let n = psrdnoise2(pos * frequency, per, rot);
        
        // Scale derivatives by frequency (as in original)
        let current_d = vec2<f32>(n.y, n.z) * frequency;
        
        // Use gradient contribution to modify amplitude
        // This is the key technique from the article - modifying contribution
        // based on accumulated derivatives
        value += amplitude * n.x / (1.0 + dot(d, d));
        
        // Accumulate scaled derivatives for next iteration
        d += current_d * amplitude;
        
        // Standard fbm updates
        amplitude *= gain;
        frequency *= lacunarity;
    }

    return value;
}

fn terrain_corrected(
    initial_pos: vec2<f32>,
    octaves: u32,
    lacunarity: f32,
    gain: f32,
    base_period: f32,
    rot: f32
) -> f32 {
    var a = 0.0;   // Accumulated noise value
    var b = 1.0;   // Amplitude for this octave
    var d = vec2<f32>(0.0, 0.0); // Accumulated derivatives
    var p = initial_pos;

    // Rotation matrix from Quilez's implementation
    let m = mat2x2<f32>(
        0.8, -0.6,
        0.6,  0.8
    );

    for (var i = 0u; i < octaves; i = i + 1u) {
        let per = vec2<f32>(base_period, base_period);
        
        // Get noise and derivatives
        let n = psrdnoise2(p, per, rot);
        
        // Add to accumulated value using gradient contribution
        a += b * n.x / (1.0 + dot(d, d));
        
        // Accumulate raw derivatives as in original
        d += vec2<f32>(n.y, n.z);
        
        // Prepare for next octave - use matrix rotation like original
        b *= gain;
        p = m * p * 2.0;
    }

    return a;
}



/// from claude

fn terrain_advanced(
    initial_pos: vec2<f32>,
    octaves: u32,
    lacunarity: f32,
    gain: f32,
    base_period: f32,
    rot: f32,
    ridge_offset: f32,  // For ridged noise
    warp_strength: f32, // For domain warping
    erosion_factor: f32 // For slope-based erosion
) -> f32 {
    // Domain warping - create more organic, flowing patterns
    var p = initial_pos;
    let warp = psrdnoise2(p * 0.5, vec2<f32>(base_period), rot);
    p += vec2<f32>(warp.y, warp.z) * warp_strength;

    var a = 0.0;   // Accumulated noise value
    var b = 1.0;   // Amplitude for this octave
    var d = vec2<f32>(0.0, 0.0); // Accumulated derivatives
    
    // Rotation matrix with slight modification for more variation
    let m = mat2x2<f32>(
        0.8, -0.6,
        0.6,  0.8
    );

    // Track weighted frequencies for erosion
    var freq_weight = 0.0;
    var total_weight = 0.0;

    for (var i = 0u; i < octaves; i = i + 1u) {
        let per = vec2<f32>(base_period, base_period);
        
        // Get noise and derivatives
        var n = psrdnoise2(p, per, rot);
        
        // Ridged noise modification
        let ridge = ridge_offset - abs(n.x);
        let square_ridge = ridge * ridge;
        
        // Combine regular and ridged noise
        let mixed_noise = mix(n.x, square_ridge, 0.5);
        
        // Enhanced gradient contribution with erosion
        let gradient_factor = 1.0 + dot(d, d);
        let erosion = 1.0 / (1.0 + erosion_factor * gradient_factor);
        
        // Weight higher frequencies more in steep areas
        let freq_contribution = pow(2.0, f32(i));
        freq_weight += mixed_noise * freq_contribution * erosion;
        total_weight += freq_contribution * erosion;
        
        // Accumulate with enhanced weighting
        a += b * mixed_noise * erosion;
        
        // Accumulate derivatives with ridge modification
        d += vec2<f32>(n.y, n.z) * sign(n.x);
        
        // Prepare for next octave
        b *= gain;
        p = m * p * 2.0;
        
        // Vary rotation slightly each octave
        let angle = 0.1 * f32(i);
        let rot_m = mat2x2<f32>(
            cos(angle), -sin(angle),
            sin(angle),  cos(angle)
        );
        p = rot_m * p;
    }

    // Blend between regular fbm and frequency-weighted version
    let weighted_fbm = freq_weight / total_weight;
    return mix(a, weighted_fbm, 0.3);
}

// Helper function to generate more varied terrains by combining multiple noise passes
fn generate_varied_terrain(
    pos: vec2<f32>,
    octaves: u32,
    base_settings: vec4<f32>, // lacunarity, gain, base_period, rot
    variation_settings: vec3<f32> // ridge_offset, warp_strength, erosion_factor
) -> f32 {
    // First pass - base terrain
    let base = terrain_advanced(
        pos,
        octaves,
        base_settings.x,
        base_settings.y,
        base_settings.z,
        base_settings.w,
        variation_settings.x,
        variation_settings.y,
        variation_settings.z
    );
    
    // Second pass - larger features with different parameters
    let large_features = terrain_advanced(
        pos * 0.5,
        octaves - 1u,
        base_settings.x * 0.8,
        base_settings.y,
        base_settings.z * 2.0,
        base_settings.w + 1.0,
        variation_settings.x * 1.2,
        variation_settings.y * 0.7,
        variation_settings.z * 0.5
    );
    
    // Third pass - detail features
    let details = terrain_advanced(
        pos * 2.0,
        octaves - 2u,
        base_settings.x * 1.2,
        base_settings.y * 0.7,
        base_settings.z * 0.5,
        base_settings.w + 2.0,
        variation_settings.x * 0.8,
        variation_settings.y * 1.3,
        variation_settings.z * 1.5
    );
    
    // Combine the passes with different weights
    return base * 0.6 + large_features * 0.3 + details * 0.1;
}