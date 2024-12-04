#define_import_path compute::utils

const PI: f32 = 3.14159;


fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    let from_range = from_max - from_min;
    let to_range = to_max - to_min;
    return (value - from_min) * to_range / from_range + to_min;
}