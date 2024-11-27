fn mod2(x: f32, y: f32) -> f32 {
    var x_mut = x;
    if abs(x) < 0.0001 {
        return 0.;
    }
    while x_mut >= y {
        x_mut = x_mut - y;
    }
    while x_mut < 0. {
        x_mut = y + x_mut;
    }
    return x_mut;
}

fn srgb_to_linear(
    coord: vec4f
) -> vec4f {
    let linear = vec4f(
        pow(coord.r, 2.2),
        pow(coord.g, 2.2),
        pow(coord.b, 2.2),
        coord.a
    );
    return linear;
}