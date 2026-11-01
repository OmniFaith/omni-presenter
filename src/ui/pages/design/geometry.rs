//! Canvas operations use a frozen press-time transform, never the previous frame's result.
use eframe::egui::{self, Pos2, Rect, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Handle { NorthWest, North, NorthEast, East, SouthEast, South, SouthWest, West, Rotate }
impl Handle {
    pub const RESIZE: [Self;8] = [Self::NorthWest,Self::North,Self::NorthEast,Self::East,Self::SouthEast,Self::South,Self::SouthWest,Self::West];
    pub fn axes(self) -> Vec2 { match self { Self::NorthWest=>egui::vec2(-1.,-1.),Self::North=>egui::vec2(0.,-1.),Self::NorthEast=>egui::vec2(1.,-1.),Self::East=>egui::vec2(1.,0.),Self::SouthEast=>egui::vec2(1.,1.),Self::South=>egui::vec2(0.,1.),Self::SouthWest=>egui::vec2(-1.,1.),Self::West=>egui::vec2(-1.,0.),Self::Rotate=>egui::vec2(0.,-1.) } }
    pub fn cursor(self) -> egui::CursorIcon { match self { Self::North|Self::South=>egui::CursorIcon::ResizeVertical,Self::East|Self::West=>egui::CursorIcon::ResizeHorizontal,Self::NorthWest|Self::SouthEast=>egui::CursorIcon::ResizeNwSe,Self::NorthEast|Self::SouthWest=>egui::CursorIcon::ResizeNeSw,Self::Rotate=>egui::CursorIcon::Crosshair } }
}

pub(super) fn rotate(vector: Vec2, degrees: f32) -> Vec2 {
    let (s,c) = degrees.to_radians().sin_cos(); egui::vec2(vector.x*c-vector.y*s, vector.x*s+vector.y*c)
}
pub(super) fn rect(canvas: Rect, bounds: [f32;4]) -> Rect {
    Rect::from_min_size(canvas.min + egui::vec2(bounds[0]*canvas.width(),bounds[1]*canvas.height()),egui::vec2(bounds[2]*canvas.width(),bounds[3]*canvas.height()))
}
pub(super) fn handle_position(bounds: Rect, rotation: f32, handle: Handle) -> Pos2 {
    let mut offset = handle.axes()*bounds.size()*0.5;
    if handle == Handle::Rotate { offset.y -= 24.0; }
    bounds.center() + rotate(offset,rotation)
}
pub(super) fn hit_handle(bounds: Rect, rotation: f32, pointer: Pos2) -> Option<Handle> {
    std::iter::once(Handle::Rotate).chain(Handle::RESIZE).find(|handle| handle_position(bounds,rotation,*handle).distance(pointer) <= 9.0)
}
pub(super) fn contains(bounds: Rect, rotation: f32, pointer: Pos2) -> bool {
    bounds.contains(bounds.center() + rotate(pointer-bounds.center(),-rotation))
}
pub(super) fn corners(bounds: Rect, rotation: f32) -> [Pos2;4] {
    [bounds.left_top(),bounds.right_top(),bounds.right_bottom(),bounds.left_bottom()].map(|p|bounds.center()+rotate(p-bounds.center(),rotation))
}
pub(super) fn snap(position: f32, length: f32, guides: &[f32], threshold: f32) -> (f32,Option<f32>) {
    guides.iter().flat_map(|guide| [position,position+length*0.5,position+length].map(move |edge| (*guide,*guide-edge)))
        .filter(|(_,delta)| delta.abs()<=threshold && position+delta>=-0.0001 && position+delta+length<=1.0001)
        .min_by(|a,b|a.1.abs().total_cmp(&b.1.abs())).map_or((position,None),|(guide,delta)|(position+delta,Some(guide)))
}
pub(super) fn resized(canvas: Rect, initial: [f32;4], rotation: f32, handle: Handle, delta: Vec2, keep_aspect: bool) -> [f32;4] {
    let bounds = rect(canvas, initial);
    let delta = rotate(delta,-rotation);
    let axes = handle.axes();
    let mut size = bounds.size();
    if axes.x != 0.0 { size.x = (size.x+delta.x*axes.x).max(8.0); }
    if axes.y != 0.0 { size.y = (size.y+delta.y*axes.y).max(8.0); }
    if keep_aspect {
        let ratio = bounds.width()/bounds.height().max(1.0);
        if axes.x == 0.0 { size.x = size.y*ratio; }
        else if axes.y == 0.0 || (size.x-bounds.width()).abs() >= (size.y-bounds.height()).abs()*ratio { size.y=size.x/ratio; }
        else { size.x=size.y*ratio; }
    }
    size.x = size.x.min(canvas.width()); size.y = size.y.min(canvas.height());
    let offset = rotate((size-bounds.size())*axes*0.5,rotation);
    let center = bounds.center()+offset;
    let min = center-size*0.5-canvas.min;
    [min.x/canvas.width(),min.y/canvas.height(),size.x/canvas.width(),size.y/canvas.height()]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn handles_resize_from_a_fixed_anchor_and_snap_chooses_the_nearest() {
        let canvas=Rect::from_min_size(Pos2::ZERO,egui::vec2(1000.,500.));
        let initial=[0.2,0.2,0.4,0.4];
        let resized=resized(canvas,initial,0.,Handle::West,egui::vec2(-50.,0.),false);
        assert!((resized[0]-0.15).abs()<0.0001);
        assert!((resized[0]+resized[2]-0.6).abs()<0.0001);
        let (x,guide)=snap(0.498,0.2,&[0.5,0.495],0.01);
        assert_eq!(guide,Some(0.5)); assert!((x-0.5).abs()<0.0001);
        for handle in Handle::RESIZE { let p=handle_position(rect(canvas,initial),35.,handle); assert_eq!(hit_handle(rect(canvas,initial),35.,p),Some(handle)); }
    }
}
