use std::sync::{Arc, Mutex};

/// Blocking dialogs and file conversion must never stop the native event loop.
pub struct BackgroundTask<T>(Arc<Mutex<Option<Result<T, String>>>>);

impl<T> Clone for BackgroundTask<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Send + 'static> BackgroundTask<T> {
    pub fn spawn_with_repaint(ctx: &eframe::egui::Context, work: impl FnOnce() -> Result<T, String> + Send + 'static) -> Self {
        let ctx = ctx.clone();
        Self::spawn(move || {
            let result = work();
            ctx.request_repaint_of(eframe::egui::ViewportId::ROOT);
            result
        })
    }

    pub fn spawn(work: impl FnOnce() -> Result<T, String> + Send + 'static) -> Self {
        let task = Self(Arc::new(Mutex::new(None)));
        let output = task.clone();
        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work))
                .unwrap_or_else(|_| Err(crate::core::i18n::Language::system().text("services-could-not-complete-the-file-operation").to_string()));
            *output.0.lock().unwrap() = Some(result);
        });
        task
    }

    pub fn take(&self) -> Option<Result<T, String>> {
        self.0.lock().unwrap().take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_blocked_file_operation_does_not_block_ui_frames() {
        let (send, receive) = std::sync::mpsc::channel();
        let task = BackgroundTask::spawn(move || {
            receive.recv().unwrap();
            Ok(42_u32)
        });
        let ctx = eframe::egui::Context::default();
        for _ in 0..5 {
            assert!(task.take().is_none());
            let mut output = ctx.run_ui(Default::default(), |ui| {
                ui.label("A interface continua respondendo");
            });
            assert!(!output.shapes.is_empty());
            output.textures_delta.clear();
        }
        send.send(()).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        loop {
            if let Some(value) = task.take() {
                assert_eq!(value.unwrap(), 42);
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
}
