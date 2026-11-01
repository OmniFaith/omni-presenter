use super::PluginShader;

pub fn wgsl(shader: &PluginShader) -> String {
    let base = include_str!("../../../assets/shaders/background.wgsl");
    let header = base.split("@fragment fn fs_main").next().unwrap();
    format!("{header}\n{}\n@fragment fn fs_main(input: Vertex) -> @location(0) vec4f {{
        let uv = input.uv;
        let aspect = u.screen.x / max(u.screen.y, 1.0);
        let point = (uv - 0.5) * vec2f(aspect, 1.0);
        let angle = u.detail.y;
        let p = vec2f(cos(angle)*point.x-sin(angle)*point.y,sin(angle)*point.x+cos(angle)*point.y)*u.motion.y;
        let t = u.screen.z * u.motion.x;
        var c = plugin_color(p, t) * u.motion.z;
        c *= 1.0 - clamp(length((uv-0.5)*1.3),0.0,1.0)*u.detail.w;
        c += vec3f((hash(uv*u.screen.xy+floor(t*12.0))-0.5)*u.finish.x);
        let pixel = uv*u.screen.xy;
        let radius = min(u.finish.y,min(u.screen.x,u.screen.y)*0.5);
        let q = abs(pixel-u.screen.xy*0.5)-u.screen.xy*0.5+vec2f(radius);
        let distance = length(max(q,vec2f(0.0)))+min(max(q.x,q.y),0.0)-radius;
        let alpha = (1.0-smoothstep(-1.0,1.0,distance))*u.finish.z;
        return vec4f(clamp(c,vec3f(0.0),vec3f(1.0))*alpha,alpha);
    }}", shader.wgsl)
}

pub fn glsl(shader: &PluginShader) -> String {
    let base = include_str!("../../../assets/shaders/background.glsl");
    let header = base.split("void main()").next().unwrap();
    format!(
        "{header}\n{}\nvoid main() {{
        float aspect=screen.x/max(screen.y,1.),t=screen.z*motion.x,angle=detail.y;
        vec2 q=(uv-.5)*vec2(aspect,1.);
        vec2 p=vec2(cos(angle)*q.x-sin(angle)*q.y,sin(angle)*q.x+cos(angle)*q.y)*motion.y;
        vec3 c=plugin_color(p,t)*motion.z;
        c*=1.-clamp(length((uv-.5)*1.3),0.,1.)*detail.w;
        c+=(hash(uv*screen.xy+floor(t*12.))-.5)*finish.x;
        pixel=vec4(clamp(c,0.,1.),1.);
    }}",
        shader.glsl
    )
}

pub fn validate(shader: &PluginShader) -> Result<(), String> {
    if shader.wgsl.len() > 65_536
        || shader.glsl.len() > 65_536
        || !shader.glsl.contains("plugin_color")
        || shader.glsl.contains("#version")
    {
        return Err("plugins-invalid-shader".into());
    }
    let source = wgsl(shader);
    let module =
        naga::front::wgsl::parse_str(&source).map_err(|error| error.emit_to_string(&source))?;
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .map_err(|error| error.to_string())?;
    if module.entry_points.len() != 2 || module.global_variables.len() != 1 {
        return Err("plugins-invalid-shader".into());
    }
    Ok(())
}
