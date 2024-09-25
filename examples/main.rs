use std::{io::Write, time::Instant};

use show_image::{create_window, ImageInfo, ImageView, WindowProxy};

use vzense_rust::{
    color_map::TURBO,
    device::{get_measuring_range, init},
    frame::{
        get_depth_mono, get_frame, get_optical_rgb, read_next_frame, Frame, FrameReady, FrameType,
    },
    touch_detector::TouchDetector,
    util::new_fixed_vec,
};

const PIX_COUNT: usize = 640 * 480;
const RUN_TOUCH_DETECTOR: bool = true;

#[show_image::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    //
    let device = match init() {
        Ok(d) => d,
        Err(msg) => {
            println!("{}", msg);
            return Ok(());
        }
    };

    let (min_depth, max_depth) = get_measuring_range(device);
    
    println!("depth range: {} mm to {} mm", min_depth, max_depth);
    println!("{}", usize::MAX);
    let optical_window = create_window("optical", Default::default())?;
    let depth_window = create_window("depth", Default::default())?;

    let mut init = true;

    let frame_ready = &mut FrameReady::default();
    let frame = &mut Frame::default();

    let signal = &mut new_fixed_vec(PIX_COUNT, 0u8);
    let rgb = &mut new_fixed_vec(3 * PIX_COUNT, 0u8);

    let mut touch_detector =
        TouchDetector::new(min_depth, max_depth, 10.0, 50.0, PIX_COUNT, 30, 10);

    let mut count = 0;
    let mut now = Instant::now();

    loop {
        read_next_frame(device, frame_ready);

        // depth //////////////////////////////////////////

        get_frame(device, frame_ready, FrameType::Depth, frame);

        if init {
            init = false;
            let w = frame.width as usize;
            let h = frame.height as usize;
            assert!(
                w * h == PIX_COUNT,
                "const PIX_COUNT not equal to {} * {}",
                w,
                h
            );
            println!("frame info: {:?}", frame);
        }

        if RUN_TOUCH_DETECTOR {
            touch_detector.process_frame(frame, count, signal);

            for (i, v) in signal.iter().enumerate() {
                for j in 0..3 {
                    rgb[3 * i + j] = *v; // black & white
                }
            }
        }

        if count < touch_detector.baseline_sample_size || !RUN_TOUCH_DETECTOR {
            // standard depth output
            get_depth_mono(frame, min_depth, max_depth, signal);

            // apply color map
            for (i, v) in signal.iter().enumerate() {
                // turbo color scale
                let one_pix_rgb = TURBO[*v as usize];
                for j in 0..3 {
                    rgb[3 * i + j] = one_pix_rgb[2 - j]; // use bgr since optical frame does too
                }
            }
        }

        update_window(&depth_window, frame.width as u32, frame.height as u32, rgb);

        // optical ////////////////////////////////////////

        get_frame(device, frame_ready, FrameType::Optical, frame);

        get_optical_rgb(frame, rgb);

        update_window(
            &optical_window,
            frame.width as u32,
            frame.height as u32,
            rgb,
        );

        count += 1;

        // update fps info
        if count % 10 == 0 {
            let elapsed = now.elapsed().as_secs_f32();
            now = Instant::now();
            print!("  fps: {:.1}\r", 10.0 / elapsed);
            std::io::stdout().flush().unwrap();
        }
    }
}

fn update_window(window: &WindowProxy, width: u32, height: u32, data: &[u8]) {
    let image = ImageView::new(ImageInfo::bgr8(width, height), data);
    window.set_image("image", image).unwrap();
}
