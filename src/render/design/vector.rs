use image::RgbaImage;
use std::sync::{Arc,OnceLock};

pub(super) fn render(svg:&str,width:u32,height:u32)->Option<RgbaImage>{
    static FONTS:OnceLock<Arc<resvg::usvg::fontdb::Database>>=OnceLock::new();
    let fonts=FONTS.get_or_init(||{let mut fonts=resvg::usvg::fontdb::Database::new();fonts.load_system_fonts();Arc::new(fonts)}).clone();
    let options=resvg::usvg::Options{fontdb:fonts,..Default::default()};
    let tree=resvg::usvg::Tree::from_str(svg,&options).ok()?;
    let mut image=resvg::tiny_skia::Pixmap::new(width.max(1),height.max(1))?;
    let scale=resvg::tiny_skia::Transform::from_scale(width as f32/tree.size().width(),height as f32/tree.size().height());
    resvg::render(&tree,scale,&mut image.as_mut());
    let mut bytes=image.take();
    // resvg returns premultiplied pixels; design source textures store straight RGBA.
    for pixel in bytes.as_chunks_mut::<4>().0{if pixel[3]>0{for i in 0..3{pixel[i]=(u32::from(pixel[i])*255/u32::from(pixel[3])).min(255)as u8;}}}
    RgbaImage::from_raw(width,height,bytes)
}
