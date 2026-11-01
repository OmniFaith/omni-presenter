#version 300 es
precision highp float;
uniform vec4 screen, motion, detail, color0, color1, color2, finish;
in vec2 uv;
out vec4 pixel;
float hash(vec2 p){return fract(sin(dot(p,vec2(127.1,311.7))+detail.z)*43758.5453);}
float noise(vec2 p){vec2 i=floor(p),f=fract(p),s=f*f*(3.-2.*f);return mix(mix(hash(i),hash(i+vec2(1,0)),s.x),mix(hash(i+vec2(0,1)),hash(i+vec2(1,1)),s.x),s.y);}
float fbm(vec2 p){float a=.5,v=0.;for(int i=0;i<6;i++){if(float(i)<clamp(detail.x*.6+1.,2.,6.))v+=noise(p)*a;p=p*2.03+vec2(17.4,8.2);a*=.5;}return v;}
vec3 palette(float a,float b){return mix(mix(color0.rgb,color1.rgb,clamp(a,0.,1.)),color2.rgb,clamp(b,0.,1.));}
void main(){
  float aspect=screen.x/max(screen.y,1.),t=screen.z*motion.x,angle=detail.y,soft=max(motion.w,.04);
  vec2 q=(uv-.5)*vec2(aspect,1.);
  vec2 p=vec2(cos(angle)*q.x-sin(angle)*q.y,sin(angle)*q.x+cos(angle)*q.y)*motion.y;
  int kind=int(screen.w);vec3 c=color0.rgb;
  if(kind==0){
    float bend=fbm(vec2(p.x*1.8+t*.15,t*.08))-.5;
    float ribbon=exp(-abs(p.y+bend*.55+sin(p.x*2.+t)*.15)/(.025+soft*.22));
    float curtain=.6+.4*sin(p.x*8.+fbm(p+t*.1)*6.);
    c=palette(ribbon*curtain,ribbon*(.5+.5*sin(p.x*1.4-t*.3))*.8);
  }else if(kind==1){
    float n=fbm(p*3.5+vec2(t*.08,-t*.04));
    c=palette(smoothstep(.15,.4+soft*.6,n),smoothstep(.4,.9,fbm(p*2.-t*.04))*.7);
    vec2 cell=floor(uv*(70.+detail.x*14.));
    float star=pow(max(0.,1.-length(fract(uv*(70.+detail.x*14.))-.5)*3.),12.);
    c+=vec3(star*step(.97,hash(cell))*(.6+.4*sin(t+hash(cell)*20.)));
  }else if(kind==2){
    float a=sin(p.x*(4.+detail.x*.5)+t+sin(p.y*3.-t*.5)),b=sin(p.y*(3.+detail.x*.5)-t*.6+p.x*1.5);
    c=palette(smoothstep(-soft,soft+.6,a+p.y),smoothstep(.2,1.3,b+a*.4)*.65);
  }else if(kind==3){
    float warp=sin(p.x*2.4+t*.5)*.8+fbm(p+t*.12)*.8;
    float silk=pow(.5+.5*sin(p.y*(8.+detail.x*2.)+warp*5.),1.2+(1.-soft)*3.);
    c=palette(silk*.9,smoothstep(.55,1.,silk)*.7);
  }else if(kind==4||kind==8){
    c=mix(color0.rgb,color1.rgb,uv.y*.15);
    for(int i=0;i<64;i++){if(float(i)<8.+detail.x*5.){
      float fi=float(i);vec2 origin=vec2(hash(vec2(fi,1)),hash(vec2(fi,4)));
      vec2 pos=vec2(origin.x+sin(t*.2+fi)*.08,fract(origin.y-t*(.025+origin.x*.025)));
      float d=length((uv-pos)*vec2(aspect,1.));
      float r=(kind==8?(.03+origin.x*.08)*(.5+soft):.008+soft*.018)/motion.y;
      c+=mix(color1.rgb,color2.rgb,origin.x)*exp(-d*d/max(r*r,.00001))*(kind==8?.3:.75);
    }}
  }else if(kind==5){
    float r=length(p+vec2(sin(t*.2),cos(t*.15))*.12);
    float ring=.5+.5*sin(r*(12.+detail.x*2.5)-t*1.4+fbm(p*2.)*soft*2.);
    c=palette(pow(ring,1.5),smoothstep(.7,1.,ring)*(.5+.5*sin(r*3.+t*.1)));
  }else if(kind==6){
    vec2 warp=vec2(fbm(p*2.+vec2(t*.1,1)),fbm(p*2.-vec2(2,t*.1)));
    float flow=fbm(p*3.+warp*(2.+soft*3.)+t*.05);
    c=palette(smoothstep(.15,.8,flow),smoothstep(.4,.9,warp.x+flow*.25));
  }else if(kind==7){
    vec2 grid=p*(5.+detail.x)+vec2(t*.12,t*.08),cell=abs(fract(grid)-.5);
    float lines=1.-smoothstep(.01,.02+soft*.12,min(cell.x,cell.y));
    c=palette(lines*(.55+.45*sin(length(p)*5.-t)),lines*(.5+.5*sin(p.x+p.y+t*.3))*.7);
  }else{
    vec2 grid=p*(3.+detail.x*.7),cell=floor(grid),local=fract(grid);
    float facet=hash(cell+step(local.x,local.y)*vec2(.5,1.7));
    float shimmer=.5+.5*sin(t*.4+facet*6.28+p.x);
    c=palette(facet*(.5+shimmer*.5),(1.-facet)*(.3+soft*.4));
  }
  c*=motion.z;
  c*=1.-clamp(length((uv-.5)*1.3),0.,1.)*detail.w;
  c+=(hash(uv*screen.xy+floor(t*12.))-.5)*finish.x;
  pixel=vec4(clamp(c,0.,1.),1.);
}
