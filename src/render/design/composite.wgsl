struct Params {
    rect: vec4<f32>,
    info: vec4<f32>,
    paint: vec4<f32>,
    tint: vec4<f32>,
    canvas: vec4<f32>,
    content: vec4<f32>,
    stroke: vec4<f32>,
    stroke_color: vec4<f32>,
    shadow: vec4<f32>,
    shadow_color: vec4<f32>,
}
@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var previous: texture_2d<f32>;
@group(0) @binding(2) var source: texture_2d<f32>;
@group(0) @binding(3) var linear_sampler: sampler;

struct Vertex { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> }
@vertex fn vs_main(@builtin(vertex_index) i:u32)->Vertex {
    let uv=vec2<f32>(f32((i<<1u)&2u),f32(i&2u));
    return Vertex(vec4<f32>(uv*vec2<f32>(2.0,-2.0)+vec2<f32>(-1.0,1.0),0.0,1.0),uv);
}

fn rotate(v:vec2<f32>,a:f32)->vec2<f32> {
    let c=cos(a); let s=sin(a);
    return vec2<f32>(c*v.x-s*v.y,s*v.x+c*v.y);
}

fn sd_polygon(p_in:vec2<f32>, verts:ptr<function,array<vec2<f32>,10>>, count:i32)->f32 {
    var d=dot(p_in-(*verts)[0],p_in-(*verts)[0]);
    var s=1.0; var j=count-1;
    for (var i=0;i<count;i=i+1) {
        let e=(*verts)[j]-(*verts)[i];
        let w=p_in-(*verts)[i];
        let b=w-e*clamp(dot(w,e)/dot(e,e),0.0,1.0);
        d=min(d,dot(b,b));
        let above = p_in.y >= (*verts)[i].y;
        let below = p_in.y < (*verts)[j].y;
        let right = e.x*w.y > e.y*w.x;
        let c = vec3<bool>(above, below, right);
        let nc = vec3<bool>(!above, !below, !right);
        if (all(c) || all(nc)) {s=-s;}
        j=i;
    }
    return s*sqrt(d);
}

fn shape_vertices(kind:f32)->array<vec2<f32>,10> {
    var v:array<vec2<f32>,10>;
    for (var i=0;i<10;i=i+1) {v[i]=vec2<f32>(0.5);}
    if kind>1.5 && kind<2.5 {v[0]=vec2<f32>(0.5,0.0);v[1]=vec2<f32>(1.0,1.0);v[2]=vec2<f32>(0.0,1.0);}
    if kind>2.5 && kind<3.5 {v[0]=vec2<f32>(0.5,0.0);v[1]=vec2<f32>(1.0,0.5);v[2]=vec2<f32>(0.5,1.0);v[3]=vec2<f32>(0.0,0.5);}
    if kind>3.5 && kind<4.5 {for (var i=0;i<10;i=i+1){let a=f32(i)*6.2831853/10.0-1.5707963;let r=select(0.22,0.5,i%2==0);v[i]=vec2<f32>(0.5+cos(a)*r,0.5+sin(a)*r);}}
    if kind>4.5 && kind<5.5 {for (var i=0;i<6;i=i+1){let a=f32(i)*6.2831853/6.0;v[i]=vec2<f32>(0.5+cos(a)*0.5,0.5+sin(a)*0.5);}}
    if kind>5.5 && kind<6.5 {v[0]=vec2<f32>(0.0,0.3);v[1]=vec2<f32>(0.65,0.3);v[2]=vec2<f32>(0.65,0.0);v[3]=vec2<f32>(1.0,0.5);v[4]=vec2<f32>(0.65,1.0);v[5]=vec2<f32>(0.65,0.7);v[6]=vec2<f32>(0.0,0.7);}
    return v;
}

fn shape_distance(local:vec2<f32>)->f32 {
    let px=vec2<f32>(max(p.content.x*p.canvas.x,1.0),max(p.content.y*p.canvas.y,1.0));
    let kind=p.content.z;
    if kind<0.5 {
        let r=min(p.paint.x*p.canvas.y/720.0,min(px.x,px.y)*0.5);
        let q=abs((local-vec2<f32>(0.5))*px)-(px*0.5-vec2<f32>(r));
        return length(max(q,vec2<f32>(0.0)))+min(max(q.x,q.y),0.0)-r;
    }
    if kind<1.5 {return (length((local-vec2<f32>(0.5))*2.0)-1.0)*min(px.x,px.y)*0.5;}
    if kind>6.5 {return abs(local.y-0.5)*px.y-max(p.content.w*p.canvas.y/720.0,1.0)*0.5;}
    var v=shape_vertices(kind);
    var count=3;
    if kind>2.5 {count=4;}
    if kind>3.5 {count=10;}
    if kind>4.5 {count=6;}
    if kind>5.5 {count=7;}
    return sd_polygon(local,&v,count)*min(px.x,px.y);
}

fn content_at(c:vec2<f32>)->vec4<f32> {
    if any(c<vec2<f32>(0.0)) || any(c>vec2<f32>(1.0)) {return vec4<f32>(0.0);}
    if p.info.w==1.0 {
        let d=shape_distance(c);
        let fill=clamp(0.5-d,0.0,1.0);
        let alpha=fill*p.tint.a;
        return vec4<f32>(p.tint.rgb*alpha,alpha);
    }
    var uv=c;
    let box_aspect=p.content.x*p.canvas.x/(p.content.y*p.canvas.y);
    let image_aspect=p.paint.y;
    if p.info.z==1.0 {
        if image_aspect>box_aspect {uv.x=(uv.x-0.5)*box_aspect/image_aspect+0.5;}
        else {uv.y=(uv.y-0.5)*image_aspect/box_aspect+0.5;}
    }
    if p.info.z==2.0 {
        if image_aspect<box_aspect {uv.x=(uv.x-0.5)*box_aspect/image_aspect+0.5;}
        else {uv.y=(uv.y-0.5)*image_aspect/box_aspect+0.5;}
        if any(uv<vec2<f32>(0.0)) || any(uv>vec2<f32>(1.0)) {return vec4<f32>(0.0);}
    }
    let t=textureSampleLevel(source,linear_sampler,uv,0.0)*p.tint;
    return vec4<f32>(t.rgb*t.a,t.a);
}

fn content_blur(c:vec2<f32>,radius:f32)->vec4<f32> {
    if radius<=0.5 {return content_at(c);}
    if p.info.w==1.0 {
        let d=shape_distance(c);
        let soft=max(radius,0.5);
        let fill=1.0-smoothstep(-soft,soft,d);
        let alpha=fill*p.tint.a;
        return vec4<f32>(p.tint.rgb*alpha,alpha);
    }
    let px=vec2<f32>(max(p.content.x*p.canvas.x,1.0),max(p.content.y*p.canvas.y,1.0));
    let inv_px=vec2<f32>(radius,radius)/px;
    var sum=content_at(c)*0.15;
    var weight=0.15;
    for (var k=0; k<20; k=k+1) {
        let fk=f32(k);
        let t=sqrt((fk+0.5)/20.0);
        let angle=fk*2.39996323;
        let offset=vec2<f32>(cos(angle),sin(angle))*(t*inv_px);
        let w=exp(-2.2*t*t);
        sum=sum+content_at(c+offset)*w;
        weight=weight+w;
    }
    return sum/weight;
}

@fragment fn fs_layer(v:Vertex)->@location(0) vec4<f32> {
    let base=textureSampleLevel(previous,linear_sampler,v.uv,0.0);
    let local=(v.uv-p.rect.xy)/max(p.rect.zw,vec2<f32>(0.00001));
    if any(local<vec2<f32>(0.0)) || any(local>vec2<f32>(1.0)) {return base;}
    let bbox_px=p.rect.zw*p.canvas.xy;
    let c=rotate((local-vec2<f32>(0.5))*bbox_px,-p.paint.w)/(vec2<f32>(max(p.content.x*p.canvas.x,1.0),max(p.content.y*p.canvas.y,1.0)))+vec2<f32>(0.5);
    let blur=max(p.paint.z,0.0)*p.canvas.y/720.0;
    let content=content_blur(c,blur);
    var color=content.rgb; var alpha=content.a;

    if p.shadow.x > 0.5 {
        let offset = rotate(vec2<f32>(p.shadow.z, p.shadow.w) * p.canvas.y / 720.0, -p.paint.w) / (vec2<f32>(max(p.content.x * p.canvas.x, 1.0), max(p.content.y * p.canvas.y, 1.0)));
        let s = content_blur(c - offset, max(p.shadow.y, 0.0) * p.canvas.y / 720.0);
        let sa = s.a * p.shadow_color.a;
        color = color + p.shadow_color.rgb * sa * (1.0 - alpha);
        alpha = alpha + sa * (1.0 - alpha);
    }
    if p.stroke.x>0.5 && p.stroke.y>0.0 {
        let width=p.stroke.y*p.canvas.y/720.0;
        var sa=0.0;
        if p.info.w==1.0 {
            let d=shape_distance(c);
            sa=clamp(0.5-(abs(d)-width*0.5),0.0,1.0)*p.stroke_color.a;
        } else {
            let px=vec2<f32>(max(p.content.x*p.canvas.x,1.0),max(p.content.y*p.canvas.y,1.0));
            let step=vec2<f32>(width,width)/px;
            var dilated=0.0;
            for (var i=0;i<8;i=i+1) {
                let a=f32(i)*0.7853982;
                dilated=max(dilated,content_at(c+vec2<f32>(cos(a),sin(a))*step*0.6).a);
            }
            sa=max(dilated-content.a,0.0)*p.stroke_color.a;
        }
        color=p.stroke_color.rgb*sa+color*(1.0-sa);
        alpha=sa+alpha*(1.0-sa);
    }
    let layer_opacity=clamp(p.info.x,0.0,1.0);
    color=color*layer_opacity;
    alpha=alpha*layer_opacity;
    if alpha<=0.001 {return base;}
    let d=base.rgb/max(base.a,0.0001);
    let s=color/max(alpha,0.0001);
    let mixed=blend(p.info.y,s,d);
    return vec4<f32>(alpha*(1.0-base.a)*s+alpha*base.a*mixed+(1.0-alpha)*base.rgb,alpha+base.a*(1.0-alpha));
}

fn blend(mode:f32,s:vec3<f32>,d:vec3<f32>)->vec3<f32> {
    if mode<0.5 {return s;}
    if mode<1.5 {return s*d;}
    if mode<2.5 {return s+d-s*d;}
    if mode<3.5 {return select(1.0-2.0*(1.0-s)*(1.0-d),2.0*s*d,d<=vec3<f32>(0.5));}
    if mode<4.5 {return min(s,d);}
    if mode<5.5 {return max(s,d);}
    if mode<6.5 {return select(min(vec3<f32>(1.0),d/max(vec3<f32>(0.0001),1.0-s)),vec3<f32>(1.0),s>=vec3<f32>(1.0));}
    if mode<7.5 {return select(vec3<f32>(0.0),1.0-min(vec3<f32>(1.0),(1.0-d)/max(vec3<f32>(0.0001),s)),s<=vec3<f32>(0.0));}
    if mode<8.5 {return select(1.0-2.0*(1.0-s)*(1.0-d),2.0*s*d,s<=vec3<f32>(0.5));}
    if mode<9.5 {let c=select(sqrt(d),((16.0*d-12.0)*d+4.0)*d,d<=vec3<f32>(0.25));return select(d+(2.0*s-1.0)*(c-d),d-(1.0-2.0*s)*d*(1.0-d),s<=vec3<f32>(0.5));}
    if mode<10.5 {return abs(s-d);}
    return s+d-2.0*s*d;
}

@fragment fn fs_display(v:Vertex)->@location(0) vec4<f32> {
    return textureSampleLevel(previous,linear_sampler,v.uv,0.0) * p.info.x;
}
