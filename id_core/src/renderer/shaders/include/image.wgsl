fn sample_image_width_height(idx: u32) -> vec2f {
    let idx_dims = idx / u32(4);
    let width = images[idx_dims];
    let height = images[idx_dims + u32(1)];
    return vec2f(f32(width), f32(height));
}

fn sample_image(idx: u32, x: f32, y: f32) -> u32 {
    let idx_dims = idx / u32(4);
    let width = images[idx_dims];
    let height = images[idx_dims + u32(1)];

    if width == u32(0) || height == u32(0) {
        discard;
    }

    let world_u = u32(mod2(x, f32(width)));
    let world_v = u32(mod2(y, f32(height)));

    let idx_u8 = idx + u32(8) + u32(2) * (world_u * height + world_v);

    var is_transparent = GET_U8(images, idx_u8);
    if is_transparent == u32(0) {
        discard;
    }

    var palette_index = GET_U8(images, idx_u8 + u32(1));
    return palette_index;
}