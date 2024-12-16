/*!
This example covers all the functionality provided by the library. It connects to a device, starts a stream, and displays the received data. The `touch_detector` is a very simple example of image processing, using depth data to detect touch events.
*/

use vzense_rust::nyx650::{
    device::{
        get_rgb_resolution, init,
        shut_down, DepthRange, RGBResolution, Resolution, DEFAULT_RESOLUTION,
    },
    // frame::{
    //     check_pixel_count, get_bgr, get_frame, get_normalized_depth, read_next_frame, Frame,
    //     FrameReady, FrameType,
    // },
    // touch_detector::TouchDetector,
};
use vzense_rust::{
    color_map::TURBO,
    util::{new_fixed_vec, KeyboardEvent},
};

use show_image::{create_window, ImageInfo, ImageView, WindowOptions, WindowProxy};
use std::{io::Write, time::Instant};

#[show_image::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    //
    // Looking for a device. If a device has been found, it will be opened and a stream started.
    let mut device = match init() {
        Ok(d) => d,
        Err(msg) => {
            println!("{}", msg);
            return Ok(());
        }
    };

    // choosing the depth range (Near, Mid, or Far)
    // set_depth_rannge(device, DepthRange::Mid);

    // let (min_depth, max_depth) = get_measuring_range(device);
    // println!("measuring range: {} mm to {} mm", min_depth, max_depth);

    // let frame_ready = &mut FrameReady::default();
    // let frame = &mut Frame::default();

    // If mapper is set to true it resets rgb_resolution to 640x480.
    // set_mapper_depth_to_rgb(device, true);

    // Setting the rgb resolution. If not set the default will be 640x480.
    // If mapper is set to true, resolution setting will be ignored and reverted to 640x480.
    // set_rgb_resolution(device, RGBResolution::RGBRes1600x1200);
    let rgb_resolution = get_rgb_resolution(device);

    let signal = &mut new_fixed_vec(DEFAULT_RESOLUTION.to_pixel_count(), 0u8);
    let out = &mut new_fixed_vec(3 * DEFAULT_RESOLUTION.to_pixel_count(), 0u8);
    let bgr = &mut new_fixed_vec(3 * rgb_resolution.to_pixel_count(), 0u8);

    // let mut touch_detector = TouchDetector::new(min_depth, max_depth, 10.0, 50.0, 30, 10);

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

    // main loop reading frames and displaying them
    // loop {
    //     read_next_frame(device, frame_ready);

    //     // depth __________________________________________

    //     get_frame(device, frame_ready, FrameType::Depth, frame);

    //     get_normalized_depth(frame, min_depth, max_depth, signal);
    //     // touch_detector.get_normalized_average_depth(signal);

    //     // apply color map
    //     for (i, si) in signal.iter().enumerate() {
    //         let rgb = &TURBO[*si as usize];
    //         out[3 * i..3 * i + 3].copy_from_slice(rgb);
    //     }

    //     update_window(&depth_window, &DEFAULT_RESOLUTION, out, Format::RGB);

    //     // touch detector _________________________________

    //     touch_detector.process(frame, signal);

    //     update_window(&touch_window, &DEFAULT_RESOLUTION, signal, Format::Mono);

    //     // rgb ____________________________________________

    //     get_frame(device, frame_ready, FrameType::RGB, frame);

    //     if init {
    //         init = false;
    //         check_pixel_count(frame, bgr.len() / 3);
    //         println!("frame info: {:?}", frame);
    //         println!("press Enter to quit")
    //     }

    //     get_bgr(frame, bgr);

    //     update_window(&rgb_window, &rgb_resolution, bgr, Format::BGR);

    //     // fps and counter info ___________________________

    //     count += 1;

    //     if count % 10 == 0 {
    //         let elapsed = now.elapsed().as_secs_f32();
    //         now = Instant::now();
    //         print!("  fps: {:.1}  frame: {}\r", 10.0 / elapsed, count);
    //         std::io::stdout().flush().unwrap();
    //     }

    //     if stop.key_was_pressed() {
    //         break;
    //     }
    // }

    stop.join();

    shut_down(&mut device);

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

// fn _destroy_window(window: &WindowProxy) {
//     window.run_function(|w| {
//         w.destroy();
//     });
// }
