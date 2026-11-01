struct Parameters {
    screen: vec4f,
    motion: vec4f,
    detail: vec4f,
    color0: vec4f,
    color1: vec4f,
    color2: vec4f,
    finish: vec4f,
};
@group(0) @binding(0) var<uniform> u: Parameters;
struct Vertex { @builtin(position) position: vec4f, @location(0) uv: vec2f };
@vertex fn vs_main(@builtin(vertex_index) index: u32) -> Vertex {
    let p = vec2f(f32((index << 1u) & 2u), f32(index & 2u));
    var out: Vertex;
    out.position = vec4f(p * vec2f(2.0, -2.0) + vec2f(-1.0, 1.0), 0.0, 1.0);
    out.uv = p;
    return out;
}
fn hash(p: vec2f) -> f32 { return fract(sin(dot(p, vec2f(127.1, 311.7)) + u.detail.z) * 43758.5453); }
fn noise(p: vec2f) -> f32 {
    let i = floor(p); let f = fract(p); let s = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2f(1.0, 0.0)), s.x), mix(hash(i + vec2f(0.0, 1.0)), hash(i + vec2f(1.0, 1.0)), s.x), s.y);
}
fn fbm(p0: vec2f) -> f32 {
    var p = p0; var amplitude = 0.5; var value = 0.0;
    for (var i = 0; i < 6; i++) {
        if f32(i) < clamp(u.detail.x * 0.6 + 1.0, 2.0, 6.0) { value += noise(p) * amplitude; }
        p = p * 2.03 + vec2f(17.4, 8.2); amplitude *= 0.5;
    }
    return value;
}
fn palette(a: f32, b: f32) -> vec3f { return mix(mix(u.color0.rgb, u.color1.rgb, clamp(a, 0.0, 1.0)), u.color2.rgb, clamp(b, 0.0, 1.0)); }
@fragment fn fs_main(in: Vertex) -> @location(0) vec4f {
    let uv = in.uv;
    let aspect = u.screen.x / max(u.screen.y, 1.0);
    let t = u.screen.z * u.motion.x;
    let angle = u.detail.y;
    let point = (uv - 0.5) * vec2f(aspect, 1.0);
    let rotated = vec2f(cos(angle) * point.x - sin(angle) * point.y, sin(angle) * point.x + cos(angle) * point.y);
    let p = rotated * u.motion.y;
    let soft = max(u.motion.w, 0.04);
    let kind = i32(u.screen.w);
    var c = u.color0.rgb;
    if kind == 0 {
        let bend = fbm(vec2f(p.x * 1.8 + t * 0.15, t * 0.08)) - 0.5;
        let ribbon = exp(-abs(p.y + bend * 0.55 + sin(p.x * 2.0 + t) * 0.15) / (0.025 + soft * 0.22));
        let curtain = 0.6 + 0.4 * sin(p.x * 8.0 + fbm(p + t * 0.1) * 6.0);
        c = palette(ribbon * curtain, ribbon * (0.5 + 0.5 * sin(p.x * 1.4 - t * 0.3)) * 0.8);
    } else if kind == 1 {
        let n = fbm(p * 3.5 + vec2f(t * 0.08, -t * 0.04));
        c = palette(smoothstep(0.15, 0.4 + soft * 0.6, n), smoothstep(0.4, 0.9, fbm(p * 2.0 - t * 0.04)) * 0.7);
        let cell = floor(uv * (70.0 + u.detail.x * 14.0));
        let star = pow(max(0.0, 1.0 - length(fract(uv * (70.0 + u.detail.x * 14.0)) - 0.5) * 3.0), 12.0);
        c += vec3f(star * step(0.97, hash(cell)) * (0.6 + 0.4 * sin(t + hash(cell) * 20.0)));
    } else if kind == 2 {
        let a = sin(p.x * (4.0 + u.detail.x * 0.5) + t + sin(p.y * 3.0 - t * 0.5));
        let b = sin(p.y * (3.0 + u.detail.x * 0.5) - t * 0.6 + p.x * 1.5);
        c = palette(smoothstep(-soft, soft + 0.6, a + p.y), smoothstep(0.2, 1.3, b + a * 0.4) * 0.65);
    } else if kind == 3 {
        let warp = sin(p.x * 2.4 + t * 0.5) * 0.8 + fbm(p + t * 0.12) * 0.8;
        let silk = pow(0.5 + 0.5 * sin(p.y * (8.0 + u.detail.x * 2.0) + warp * 5.0), 1.2 + (1.0 - soft) * 3.0);
        c = palette(silk * 0.9, smoothstep(0.55, 1.0, silk) * 0.7);
    } else if kind == 4 || kind == 8 {
        c = mix(u.color0.rgb, u.color1.rgb, uv.y * 0.15);
        for (var i = 0; i < 64; i++) {
            if f32(i) < 8.0 + u.detail.x * 5.0 {
                let fi = f32(i);
                let origin = vec2f(hash(vec2f(fi, 1.0)), hash(vec2f(fi, 4.0)));
                let position = vec2f(origin.x + sin(t * 0.2 + fi) * 0.08, fract(origin.y - t * (0.025 + origin.x * 0.025)));
                let distance = length((uv - position) * vec2f(aspect, 1.0));
                let radius = select(0.008 + soft * 0.018, (0.03 + origin.x * 0.08) * (0.5 + soft), kind == 8) / u.motion.y;
                let glow = exp(-distance * distance / max(radius * radius, 0.00001));
                c += mix(u.color1.rgb, u.color2.rgb, origin.x) * glow * select(0.75, 0.3, kind == 8);
            }
        }
    } else if kind == 5 {
        let radius = length(p + vec2f(sin(t * 0.2), cos(t * 0.15)) * 0.12);
        let ring = 0.5 + 0.5 * sin(radius * (12.0 + u.detail.x * 2.5) - t * 1.4 + fbm(p * 2.0) * soft * 2.0);
        c = palette(pow(ring, 1.5), smoothstep(0.7, 1.0, ring) * (0.5 + 0.5 * sin(radius * 3.0 + t * 0.1)));
    } else if kind == 6 {
        let warp = vec2f(fbm(p * 2.0 + vec2f(t * 0.1, 1.0)), fbm(p * 2.0 - vec2f(2.0, t * 0.1)));
        let flow = fbm(p * 3.0 + warp * (2.0 + soft * 3.0) + t * 0.05);
        c = palette(smoothstep(0.15, 0.8, flow), smoothstep(0.4, 0.9, warp.x + flow * 0.25));
    } else if kind == 7 {
        let grid = p * (5.0 + u.detail.x) + vec2f(t * 0.12, t * 0.08);
        let cell = abs(fract(grid) - 0.5);
        let lines = 1.0 - smoothstep(0.01, 0.02 + soft * 0.12, min(cell.x, cell.y));
        let pulse = 0.55 + 0.45 * sin(length(p) * 5.0 - t);
        c = palette(lines * pulse, lines * (0.5 + 0.5 * sin(p.x + p.y + t * 0.3)) * 0.7);
    } else {
        let grid = p * (3.0 + u.detail.x * 0.7);
        let cell = floor(grid);
        let local = fract(grid);
        let triangle = step(local.x, local.y);
        let facet = hash(cell + triangle * vec2f(0.5, 1.7));
        let shimmer = 0.5 + 0.5 * sin(t * 0.4 + facet * 6.28 + p.x);
        c = palette(facet * (0.5 + shimmer * 0.5), (1.0 - facet) * (0.3 + soft * 0.4));
    }
    c *= u.motion.z;
    c *= 1.0 - clamp(length((uv - 0.5) * 1.3), 0.0, 1.0) * u.detail.w;
    c += (hash(uv * u.screen.xy + floor(t * 12.0)) - 0.5) * u.finish.x;
    let r = min(u.finish.y, min(u.screen.x, u.screen.y) * 0.5);
    let q = abs((uv - 0.5) * u.screen.xy) - u.screen.xy * 0.5 + r;
    let distance = length(max(q, vec2f(0.0))) + min(max(q.x, q.y), 0.0) - r;
    let alpha = (1.0 - smoothstep(-0.7, 0.7, distance)) * u.finish.z;
    return vec4f(clamp(c, vec3f(0.0), vec3f(1.0)) * alpha, alpha);
}
