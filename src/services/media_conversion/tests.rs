use super::*;

#[test]
fn converts_images_audio_and_video_without_modifying_originals() {
    if Command::new("ffmpeg").arg("-version").output().is_err() {
        return;
    }
    let root = std::env::temp_dir().join(crate::core::preferences::new_id("media-conversion-test"));
    std::fs::create_dir(&root).unwrap();
    let image = root.join("source.png");
    image::RgbaImage::from_pixel(8, 8, image::Rgba([25, 100, 200, 150]))
        .save(&image)
        .unwrap();
    let audio = root.join("sound.wav");
    assert!(
        Command::new("ffmpeg")
            .args([
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=0.1",
                "-y"
            ])
            .arg(&audio)
            .status()
            .unwrap()
            .success()
    );
    let video = root.join("video.mkv");
    assert!(
        Command::new("ffmpeg")
            .args([
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "testsrc=size=17x19:rate=5:duration=0.2",
                "-c:v",
                "ffv1",
                "-y"
            ])
            .arg(&video)
            .status()
            .unwrap()
            .success()
    );
    let originals: Vec<_> = [&image, &audio, &video]
        .into_iter()
        .map(|p| std::fs::read(p).unwrap())
        .collect();
    let paths = vec![image.clone(), audio.clone(), video.clone()];
    let first = convert(paths.clone(), Formats::default(), &root.join("output")).unwrap();
    assert_eq!(
        first
            .iter()
            .map(|p| p.extension().unwrap().to_str().unwrap())
            .collect::<Vec<_>>(),
        ["png", "flac", "mp4"]
    );
    assert_eq!(
        image::open(&first[0]).unwrap().to_rgba8().get_pixel(0, 0).0,
        [25, 100, 200, 150]
    );
    let second = convert(
        paths.clone(),
        Formats {
            image: ImageFormat::Webp,
            audio: AudioFormat::Mp3,
            video: VideoFormat::LosslessMkv,
            video_resolution: VideoResolution::Original,
            video_aspect_ratio: VideoAspectRatio::Original,
        },
        &root.join("output"),
    )
    .unwrap();
    assert_eq!(
        second
            .iter()
            .map(|p| p.extension().unwrap().to_str().unwrap())
            .collect::<Vec<_>>(),
        ["webp", "mp3", "mkv"]
    );
    assert_eq!(
        image::open(&second[0])
            .unwrap()
            .to_rgba8()
            .get_pixel(0, 0)
            .0,
        [25, 100, 200, 150]
    );
    for (path, before) in paths.iter().zip(originals) {
        assert_eq!(std::fs::read(path).unwrap(), before);
    }
    for path in first.iter().chain(second.iter()) {
        assert!(
            Command::new("ffprobe")
                .args([
                    "-v",
                    "error",
                    "-show_entries",
                    "format=format_name",
                    "-of",
                    "default=nw=1"
                ])
                .arg(path)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_conversion_does_not_leave_partial_files_or_change_originals() {
    if Command::new("ffmpeg").arg("-version").output().is_err() {
        return;
    }
    let root =
        std::env::temp_dir().join(crate::core::preferences::new_id("media-conversion-failure"));
    std::fs::create_dir(&root).unwrap();
    let source = root.join("broken.jpg");
    std::fs::write(&source, b"invalid image").unwrap();
    assert!(
        convert(
            vec![source.clone()],
            Formats::default(),
            &root.join("output")
        )
        .is_err()
    );
    assert_eq!(std::fs::read(source).unwrap(), b"invalid image");
    assert_eq!(std::fs::read_dir(root.join("output")).unwrap().count(), 0);
    std::fs::remove_dir_all(root).unwrap();
}
