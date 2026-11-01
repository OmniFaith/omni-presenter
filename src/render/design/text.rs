//! Rasterize at the output's physical resolution and align complete lines, not glyph ink boxes.
use crate::core::design::{BlendMode, DesignLayer};
use fontdue::{Font, FontSettings};
use image::{Rgba, RgbaImage};
use std::{collections::HashMap, sync::{Arc, Mutex, OnceLock}};

fn font(layer:&DesignLayer)->Option<Arc<Font>> {
    static FONTS:OnceLock<Mutex<HashMap<String,Arc<Font>>>>=OnceLock::new();
    let cache=FONTS.get_or_init(Mutex::default);
    let key=format!("{}:{}:{}",layer.font_family,layer.effective_font_weight(),layer.text_style.italic);
    if let Some(font)=cache.lock().unwrap().get(&key).cloned(){return Some(font);}
    let bytes=crate::core::models::fonts::presentation_font_bytes(&layer.font_family,layer.effective_font_weight(),layer.text_style.italic);
    let font=Arc::new(Font::from_bytes(bytes,FontSettings::default()).ok()?);
    cache.lock().unwrap().insert(key,font.clone());Some(font)
}
fn cased(text:&str,case:&str)->String {match case{
    "upper"=>text.to_uppercase(),"lower"=>text.to_lowercase(),"title"=>text.split_inclusive(char::is_whitespace).map(|word|{let mut chars=word.chars();chars.next().map_or_else(String::new,|first|first.to_uppercase().collect::<String>()+chars.as_str())}).collect(),_=>text.into()
}}
fn measure(font:&Font,chars:&[char],size:f32,spacing:f32)->f32{
    let mut width=0.;let mut previous=None;
    for &c in chars {if let Some(p)=previous{width+=font.horizontal_kern(p,c,size).unwrap_or(0.)+spacing;}width+=font.metrics(c,size).advance_width;previous=Some(c);}
    width
}
struct Line{chars:Vec<char>,paragraph_end:bool}
fn lines(font:&Font,text:&str,size:f32,spacing:f32,width:f32,wrap:bool)->Vec<Line>{
    let mut result=Vec::new();
    for paragraph in text.split('\n') {
        let mut current:Vec<char>=Vec::new();
        for c in paragraph.chars(){
            current.push(c);
            if wrap && current.len()>1 && measure(font,&current,size,spacing)>width {
                let cut=current[..current.len()-1].iter().rposition(|c|c.is_whitespace()).filter(|i|*i>0).unwrap_or(current.len()-1);
                let mut remainder=current.split_off(cut);
                while current.last().is_some_and(|c|c.is_whitespace()){current.pop();}
                while remainder.first().is_some_and(|c|c.is_whitespace()){remainder.remove(0);}
                result.push(Line{chars:current,paragraph_end:false});current=remainder;
            }
        }
        result.push(Line{chars:current,paragraph_end:true});
    }
    result
}

pub(super) fn draw(image:&mut RgbaImage,text:&str,layer:&DesignLayer,scale:f32){
    let Some(font)=font(layer)else{return;};
    let text=cased(text,&layer.text_style.case);
    let spacing=layer.text_style.letter_spacing*scale;
    let paragraph_gap=layer.text_style.paragraph_spacing.max(0.)*scale;
    let mut size=(layer.font_size*scale).clamp(1.,2048.);
    let mut layout=lines(&font,&text,size,spacing,image.width()as f32,layer.text_style.wrap);
    let height_of=|lines:&[Line],size:f32|{
        lines.len()as f32*size*layer.text_style.line_height.max(0.5)
            +lines.iter().take(lines.len().saturating_sub(1)).filter(|line|line.paragraph_end).count()as f32*paragraph_gap
    };
    if layer.text_style.auto_fit {
        for _ in 0..20{
            let height=height_of(&layout,size);
            let width=layout.iter().map(|line|measure(&font,&line.chars,size,spacing)).fold(0.,f32::max);
            if (height<=image.height()as f32 && width<=image.width()as f32)||size<=2.{break;}
            size=(size*(image.height()as f32/height.max(1.)).min(image.width()as f32/width.max(1.)).min(0.95)).max(2.);
            layout=lines(&font,&text,size,spacing,image.width()as f32,layer.text_style.wrap);
        }
    }
    let total=height_of(&layout,size);
    let mut top=match layer.text_style.vertical.as_str(){"top"=>0.,"bottom"=>image.height()as f32-total,_=>(image.height()as f32-total)*0.5};
    let metrics=font.horizontal_line_metrics(size);
    let ascent=metrics.map_or(size*0.8,|m|m.ascent);
    let descent=metrics.map_or(-size*0.2,|m|m.descent);
    let line_height=size*layer.text_style.line_height.max(0.5);
    for line in &layout{
        let width=measure(&font,&line.chars,size,spacing);
        let left=match layer.align.as_str(){"left"=>0.,"right"=>image.width()as f32-width,_=>(image.width()as f32-width)*0.5};
        let baseline=top+(line_height-(ascent-descent))*0.5+ascent;
        let mut pen=left;let mut previous=None;
        for &c in &line.chars{
            if let Some(p)=previous{pen+=font.horizontal_kern(p,c,size).unwrap_or(0.)+spacing;}
            let (glyph,bitmap)=font.rasterize(c,size);
            let x=(pen+glyph.xmin as f32).round()as i32;
            let y=(baseline-glyph.height as f32-glyph.ymin as f32).round()as i32;
            for py in 0..glyph.height{for px in 0..glyph.width{
                let dx=x+px as i32;let dy=y+py as i32;
                if dx<0||dy<0||dx>=image.width()as i32||dy>=image.height()as i32{continue;}
                let mut color=layer.color;color[3]=(u32::from(color[3])*u32::from(bitmap[py*glyph.width+px])/255)as u8;
                super::raster::composite(image.get_pixel_mut(dx as u32,dy as u32),Rgba(color),1.,BlendMode::Normal);
            }}
            pen+=glyph.advance_width;previous=Some(c);
        }
        for (enabled,offset) in [(layer.text_style.underline,size*0.1),(layer.text_style.strike,-size*0.3)]{
            if !enabled{continue;}
            for y in (baseline+offset).round()as i32..(baseline+offset+(size*0.05).max(1.)).ceil()as i32{
                for x in left.round()as i32..(left+width).round()as i32{
                    if x>=0&&y>=0&&x<image.width()as i32&&y<image.height()as i32{super::raster::composite(image.get_pixel_mut(x as u32,y as u32),Rgba(layer.color),1.,BlendMode::Normal);}
                }
            }
        }
        top+=line_height+if line.paragraph_end{paragraph_gap}else{0.};
    }
}
