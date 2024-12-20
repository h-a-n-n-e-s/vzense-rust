/*!
This example covers all the functionality provided by the library. It connects to a device, starts a stream, and displays the received data. The `touch_detector` is a very simple example of image processing, using depth data to detect touch events.
*/

// by default using the newest Scepter API
#[cfg(not(feature = "dcam560"))]
use vzense_rust::scepter::{
    device::{get_rgb_resolution, init, map_rgb_to_depth, set_rgb_resolution, shut_down},
    frame::{
        get_bgr, get_frame, get_normalized_depth, read_next_frame, Frame, FrameReady, FrameType,
    },
};

// uses the older API specifically for the DCAM560 model
#[cfg(feature = "dcam560")]
use vzense_rust::dcam560::{
    device::{
        get_depth_measuring_range, get_rgb_resolution, init, map_rgb_to_depth,
        set_depth_measuring_range_dcam560, set_rgb_resolution, shut_down, DepthRange,
    },
    frame::{
        check_pixel_count, get_bgr, get_frame, get_normalized_depth, read_next_frame, Frame,
        FrameReady, FrameType,
    },
};

use vzense_rust::util::{
    color_map::TURBO, new_fixed_vec, touch_detector::TouchDetector, KeyboardEvent, RGBResolution,
    Resolution, DEFAULT_PIXEL_COUNT, DEFAULT_RESOLUTION,
};

use show_image::{ImageInfo, ImageView, WindowOptions, WindowProxy};
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

    // choosing the depth measuring range for DCAM560 (Near, Mid, or Far)
    #[cfg(feature = "dcam560")]
    {
        set_depth_measuring_range_dcam560(&device, DepthRange::Mid);

        let range = get_depth_measuring_range(&device);
        println!("depth measuring range: {} mm to {} mm", range.0, range.1);
    }

    // choosing the min/max depth in mm for the color mapping of the depth output. These values also bound the depths used in the `TochDetector` to reduce measuring artifacts. In the specs the depth measuring range for the NYX650 is given as min: 300 mm, max: 4500 mm. The depth measuring range for the DCAM560 depends on the range chosen above.
    let (min_depth, max_depth) = (500, 1500);

    let mut touch_detector =
        TouchDetector::new(min_depth, max_depth, 5.0, 50.0, 30, 10, DEFAULT_PIXEL_COUNT);

    // Setting the rgb resolution. If not set the default will be 640x480.
    // If mapper is set to true, resolution setting will be ignored and reverted to 640x480.
    set_rgb_resolution(&device, RGBResolution::RGBRes640x480);

    let mut rgb_frame_type = FrameType::RGB;

    // If map_rgb is set to true rgb_resolution is reset to 640x480.
    let map_rgb = true;
    if map_rgb {
        rgb_frame_type = FrameType::RGBMapped;
        map_rgb_to_depth(&device, map_rgb);
    }

    let rgb_resolution = get_rgb_resolution(&device);

    let signal = &mut new_fixed_vec(DEFAULT_PIXEL_COUNT, 0u8);
    let out = &mut new_fixed_vec(3 * DEFAULT_PIXEL_COUNT, 0u8);
    let bgr = &mut new_fixed_vec(3 * rgb_resolution.to_pixel_count(), 0u8);

    // creating the image windows, `double()` doubles the size of the window in both dimensions
    let depth_window = create_window("depth", &DEFAULT_RESOLUTION.double(), true);
    let touch_window = create_window("touch", &DEFAULT_RESOLUTION.double(), true);
    let rgb_window = create_window("rgb", &rgb_resolution.double(), true);

    let stop = KeyboardEvent::new("\n");

    let frame_ready = &mut FrameReady::default();
    let frame = &mut Frame::default();

    let mut count = 0;
    let mut now = Instant::now();
    let mut init = true;

    // main loop reading frames and displaying them
    loop {
        // Scepter API has an additional `max_wait_time_ms` paramter
        #[cfg(not(feature = "dcam560"))]
        read_next_frame(&device, 1200, frame_ready);

        #[cfg(feature = "dcam560")]
        read_next_frame(&device, frame_ready);

        // depth __________________________________________

        get_frame(&device, frame_ready, &FrameType::Depth, frame);

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

        get_frame(&device, frame_ready, &rgb_frame_type, frame);

        if init {
            init = false;
            // check_pixel_count(frame, bgr.len() / 3);
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

fn create_window(name: &str, size: &Resolution, allow_drag_and_zoom: bool) -> WindowProxy {
    show_image::create_window(
        name,
        WindowOptions {
            size: Some(size.to_array()),
            default_controls: allow_drag_and_zoom,
            ..Default::default()
        },
    )
    .unwrap()
}

// fn _destroy_window(window: &WindowProxy) {
//     window.run_function(|w| {
//         w.destroy();
//     });
// }
