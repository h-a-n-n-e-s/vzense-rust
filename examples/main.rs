use std::{io::Write, time::Instant};

use show_image::{create_window, ImageInfo, ImageView, WindowOptions, WindowProxy};

use vzense_rust::{
    color_map::TURBO,
    device::{
        get_measuring_range, get_rgb_resolution, init, set_rgb_resolution, shut_down, RGBResolution, Resolution, DEFAULT_RESOLUTION
    },
    frame::{
        check_pixel_count, get_bgr, get_frame, get_normalized_depth, read_next_frame, Frame,
        FrameReady, FrameType,
    },
    touch_detector::TouchDetector,
    util::{new_fixed_vec, KeyboardEvent},
};

#[show_image::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    //
    let mut device = match init() {
        Ok(d) => d,
        Err(msg) => {
            println!("{}", msg);
            return Ok(());
        }
    };

    let (min_depth, max_depth) = get_measuring_range(device);

    println!("depth range: {} mm to {} mm", min_depth, max_depth);

    let frame_ready = &mut FrameReady::default();
    let frame = &mut Frame::default();

    set_rgb_resolution(device, RGBResolution::RGBRes1600x1200);
    let rgb_resolution = get_rgb_resolution(device);

    let signal = &mut new_fixed_vec(DEFAULT_RESOLUTION.to_pixel_count(), 0u8);
    let out = &mut new_fixed_vec(3 * DEFAULT_RESOLUTION.to_pixel_count(), 0u8);
    let bgr = &mut new_fixed_vec(3 * rgb_resolution.to_pixel_count(), 0u8);

    let mut touch_detector = TouchDetector::new(min_depth, max_depth, 10.0, 50.0, 30, 10);

    let rgb_window = create_window(
        "rgb",
        WindowOptions {
            size: Some(rgb_resolution.to_array()),
            ..Default::default()
        },
    )?;
    let depth_window = create_window("depth", Default::default())?;
    let touch_window = create_window("touch", Default::default())?;

    let stop = KeyboardEvent::new("\n");

    let mut count = 0;
    let mut now = Instant::now();
    let mut init = true;

    loop {
        read_next_frame(device, frame_ready);

        // depth __________________________________________

        get_frame(device, frame_ready, FrameType::Depth, frame);

        get_normalized_depth(frame, min_depth, max_depth, signal);
        // touch_detector.get_normalized_average_depth(signal);

        // apply color map
        for (i, si) in signal.iter().enumerate() {
            let rgb = &TURBO[*si as usize];
            out[3 * i..3 * i + 3].copy_from_slice(rgb);
        }

        update_window(&depth_window, &DEFAULT_RESOLUTION, out, Format::RGB);

        // touch detector _________________________________

        touch_detector.process(frame, signal);

        update_window(&touch_window, &DEFAULT_RESOLUTION, signal, Format::Mono);

        // rgb ____________________________________________

        get_frame(device, frame_ready, FrameType::RGB, frame);

        if init {
            init = false;
            check_pixel_count(frame, bgr.len() / 3);
            println!("frame info: {:?}", frame);
            println!("press Enter to quit")
        }

        get_bgr(frame, bgr);

        update_window(&rgb_window, &rgb_resolution, bgr, Format::BGR);

        // fps and counter info ___________________________

        count += 1;

        if count % 10 == 0 {
            let elapsed = now.elapsed().as_secs_f32();
            now = Instant::now();
            print!("  fps: {:.1}  frame: {}\r", 10.0 / elapsed, count);
            std::io::stdout().flush().unwrap();
        }

        if stop.key_was_pressed() {
            break;
        }
    }

    stop.join();

    shut_down(&mut device);

    // rgb_window.run_function(|w| {
    //     w.destroy();
    // });

    Ok(())

}

enum Format {
    Mono,
    RGB,
    BGR,
}
fn update_window(window: &WindowProxy, resolution: &Resolution, data: &[u8], format: Format) {
    let (w, h) = resolution.to_tuple();
    let info = match format {
        Format::Mono => ImageInfo::mono8(w, h),
        Format::RGB => ImageInfo::rgb8(w, h),
        Format::BGR => ImageInfo::bgr8(w, h),
    };
    let image = ImageView::new(info, data);
    window.set_image("image", image).unwrap();
}
