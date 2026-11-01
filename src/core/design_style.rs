use serde::{Deserialize,Serialize};

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct LayerBackground { pub enabled:bool, pub color:[u8;4], pub radius:f32, pub padding:f32 }
impl Default for LayerBackground {fn default()->Self {Self {enabled:false,color:[15,23,42,200],radius:12.,padding:0.}}}

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq,Serialize,Deserialize)]
pub enum ShapeKind { #[default] Rectangle,Ellipse,Triangle,Diamond,Star,Hexagon,Arrow,Line }
impl ShapeKind {
    pub const ALL:[Self;8]=[Self::Rectangle,Self::Ellipse,Self::Triangle,Self::Diamond,Self::Star,Self::Hexagon,Self::Arrow,Self::Line];
    pub fn label(self)->&'static str {match self {Self::Rectangle=>"design-rectangle",Self::Ellipse=>"design-ellipse",Self::Triangle=>"design-triangle",Self::Diamond=>"design-diamond",Self::Star=>"design-star",Self::Hexagon=>"design-hexagon",Self::Arrow=>"design-arrow",Self::Line=>"design-line"}}
    pub fn polygon(self)->Vec<[f32;2]> {
        match self {
            Self::Triangle=>vec![[0.5,0.0],[1.0,1.0],[0.0,1.0]],
            Self::Diamond=>vec![[0.5,0.0],[1.0,0.5],[0.5,1.0],[0.0,0.5]],
            Self::Arrow=>vec![[0.0,0.3],[0.65,0.3],[0.65,0.0],[1.0,0.5],[0.65,1.0],[0.65,0.7],[0.0,0.7]],
            Self::Hexagon=>(0..6).map(|i|{let a=i as f32*std::f32::consts::TAU/6.0;[0.5+a.cos()*0.5,0.5+a.sin()*0.5]}).collect(),
            Self::Star=>(0..10).map(|i|{let a=i as f32*std::f32::consts::TAU/10.0-std::f32::consts::FRAC_PI_2;let r=if i%2==0{0.5}else{0.22};[0.5+a.cos()*r,0.5+a.sin()*r]}).collect(),
            _=>Vec::new(),
        }
    }
}

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct LayerStroke {pub enabled:bool,pub width:f32,pub color:[u8;4]}
impl Default for LayerStroke {fn default()->Self{Self{enabled:false,width:2.0,color:[255;4]}}}

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct LayerShadow {pub enabled:bool,pub blur:f32,pub x:f32,pub y:f32,pub color:[u8;4]}
impl Default for LayerShadow {fn default()->Self{Self{enabled:false,blur:12.0,x:0.0,y:8.0,color:[0,0,0,150]}}}

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct DesignTextStyle {
    pub italic:bool,pub underline:bool,pub strike:bool,pub line_height:f32,pub letter_spacing:f32,
    pub paragraph_spacing:f32,pub vertical:String,pub case:String,pub wrap:bool,pub auto_fit:bool,
}
impl Default for DesignTextStyle {
    fn default()->Self {Self{italic:false,underline:false,strike:false,line_height:1.2,letter_spacing:0.0,
        paragraph_spacing:0.0,vertical:"center".into(),case:"original".into(),wrap:true,auto_fit:true}}
}
