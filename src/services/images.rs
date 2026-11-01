//! Decode, resize and prepare image pixels off the UI/event-loop thread.
use eframe::egui;
use image::RgbaImage;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock, mpsc},
    time::Duration,
};

#[derive(Clone)]
pub struct DecodedImage {
    pub rgba: Arc<RgbaImage>,
    pub color: Arc<egui::ColorImage>,
}

#[derive(Clone)]
pub enum ImageLoad {
    Pending,
    Ready(DecodedImage),
    Failed(String),
}

type Key = (String, u32);
type Cache = Arc<Mutex<HashMap<Key, ImageLoad>>>;
struct Job {
    key: Key,
    ctx: egui::Context,
    viewport: egui::ViewportId,
}
struct Loader {
    cache: Cache,
    queue: mpsc::Sender<Job>,
}

fn loader() -> &'static Loader {
    static LOADER: OnceLock<Loader> = OnceLock::new();
    LOADER.get_or_init(|| {
        let cache: Cache = Arc::default();
        let (queue, receive) = mpsc::channel::<Job>();
        let receive = Arc::new(Mutex::new(receive));
        for _ in 0..2 {
            let receive = receive.clone();
            let cache = cache.clone();
            std::thread::spawn(move || {
                loop {
                    let job = { receive.lock().unwrap().recv() };
                    let Ok(job) = job else {
                        break;
                    };
                    let result = decode(&job.key.0, job.key.1)
                        .map(ImageLoad::Ready)
                        .unwrap_or_else(ImageLoad::Failed);
                    {
                        let mut cache = cache.lock().unwrap();
                        let bytes = |value: &ImageLoad| match value {
                            ImageLoad::Ready(image) => {
                                image.rgba.as_raw().len() + image.color.pixels.len() * 4
                            }
                            _ => 0,
                        };
                        let incoming = bytes(&result);
                        let mut used: usize = cache.values().map(bytes).sum();
                        while used + incoming > 256 * 1024 * 1024 {
                            let evict = cache.iter().find_map(|(key, value)| {
                                matches!(value, ImageLoad::Ready(_)).then_some(key.clone())
                            });
                            let Some(evict) = evict else {
                                break;
                            };
                            if let Some(value) = cache.remove(&evict) {
                                used -= bytes(&value);
                            }
                        }
                        cache.insert(job.key, result);
                    }
                    job.ctx.request_repaint_of(job.viewport);
                    job.ctx.request_repaint_of(egui::ViewportId::ROOT);
                }
            });
        }
        Loader { cache, queue }
    })
}

fn decode(path: &str, max_side: u32) -> Result<DecodedImage, String> {
    let mut reader = image::ImageReader::open(path)
        .map_err(|e| e.to_string())?
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(256 * 1024 * 1024);
    limits.max_image_width = Some(32768);
    limits.max_image_height = Some(32768);
    reader.limits(limits);
    let source = reader.decode().map_err(|e| e.to_string())?;
    let source = if source.width().max(source.height()) > max_side {
        source.thumbnail(max_side, max_side)
    } else {
        source
    };
    let rgba = source.into_rgba8();
    let color = egui::ColorImage::from_rgba_unmultiplied(
        [rgba.width() as usize, rgba.height() as usize],
        rgba.as_raw(),
    );
    Ok(DecodedImage {
        rgba: Arc::new(rgba),
        color: Arc::new(color),
    })
}

pub fn request(ctx: &egui::Context, path: &str, max_side: u32) -> ImageLoad {
    let loader = loader();
    let key = (path.to_owned(), max_side.clamp(16, 4096));
    let state = {
        let mut cache = loader.cache.lock().unwrap();
        if let Some(state) = cache.get(&key) {
            state.clone()
        } else {
            cache.insert(key.clone(), ImageLoad::Pending);
            if loader
                .queue
                .send(Job {
                    key: key.clone(),
                    ctx: ctx.clone(),
                    viewport: ctx.viewport_id(),
                })
                .is_err()
            {
                cache.insert(
                    key.clone(),
                    ImageLoad::Failed(crate::core::i18n::Language::system().text("services-could-not-load-the-image").to_string()),
                );
            }
            ImageLoad::Pending
        }
    };
    if matches!(state, ImageLoad::Pending) {
        ctx.request_repaint_after(Duration::from_millis(33));
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_requests_do_not_decode_on_the_ui_thread_and_report_bad_files() {
        let ctx = egui::Context::default();
        let path = std::env::temp_dir().join(format!(
            "omni-presenter-async-image-{}.png",
            crate::core::preferences::new_id("test")
        ));
        image::RgbaImage::from_pixel(640, 480, image::Rgba([80, 100, 120, 255]))
            .save(&path)
            .unwrap();
        let file = path.to_string_lossy();
        let start = std::time::Instant::now();
        assert!(matches!(request(&ctx, &file, 320), ImageLoad::Pending));
        assert!(start.elapsed() < Duration::from_millis(100));
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        loop {
            match request(&ctx, &file, 320) {
                ImageLoad::Ready(image) => {
                    assert!(image.rgba.width().max(image.rgba.height()) <= 320);
                    assert_eq!(
                        image.color.size,
                        [image.rgba.width() as usize, image.rgba.height() as usize]
                    );
                    break;
                }
                ImageLoad::Failed(error) => panic!("{error}"),
                ImageLoad::Pending => {
                    assert!(std::time::Instant::now() < deadline);
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }
        std::fs::remove_file(&path).unwrap();
        let missing = "missing-image-regression-test.png";
        loop {
            match request(&ctx, missing, 320) {
                ImageLoad::Failed(error) => {
                    assert!(!error.is_empty());
                    break;
                }
                ImageLoad::Pending => {
                    assert!(std::time::Instant::now() < deadline);
                    std::thread::sleep(Duration::from_millis(10));
                }
                ImageLoad::Ready(_) => panic!("Missing file must fail"),
            }
        }
    }
}
