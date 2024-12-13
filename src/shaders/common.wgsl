#define_import_path compute::common

struct Params {
    dimensions: u32,

    // circle generator
    radius: f32,
    noise_seed: u32,
    noise_freq: f32,
    noise_amplitude: f32,
    noise_offset: f32,
    noise_octaves: i32,
    noise_lacunarity:f32,
    power_bias: f32,
    flatness: f32,
    steepness: f32,
    mix: f32,
    noise_warp_amount: f32,
    noise_warp_scale: f32,

    // domain warp 1
    domain_warp_1_amount_a: f32,
    domain_warp_1_scale_a: f32,
    domain_warp_1_amount_b: f32,
    domain_warp_1_scale_b: f32,
    
    // cellular automata
    noise_weight: f32,
    ca_thresh: f32,
    ca_search_radius: f32,
    ca_edge_pow: f32,
    edge_suppress_mix: f32,

    // cave domain warp
    domain_warp_2_amount_a: f32,
    domain_warp_2_scale_a: f32,
    domain_warp_2_amount_b: f32,
    domain_warp_2_scale_b: f32,

    misc_f: f32,
    misc_i: i32,
}

const BUFFER_LEN = 1024u;
const GRID_SIZE = 8u;

const STRIP_SIZE = 8192u;
const STRIP_COUNT = 3u;

struct DataGrid{
    floats: array<array<array<f32, GRID_SIZE>, BUFFER_LEN>, BUFFER_LEN>,
    ints: array<array<array<i32, GRID_SIZE>, BUFFER_LEN>, BUFFER_LEN>,
};

struct DataStrip{
    floats: array<array<f32, STRIP_SIZE>, STRIP_COUNT>,
    ints: array<array<i32, STRIP_SIZE>, STRIP_COUNT>,
};

