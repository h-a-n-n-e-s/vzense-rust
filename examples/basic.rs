/*!
This example covers all the functionality provided by the library. It connects to a device, starts a stream, and displays the received data. The `touch_detector` is a very simple example of image processing, using depth data to detect touch events.
*/

// by default using the newest Scepter API
#[cfg(not(feature = "dcam560"))]
use vzense_rust::scepter::{
    device::{ColorFormat, Device},
    frame::{get_frame, read_next_frame, FrameType},
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
    color_map::TURBO, new_fixed_vec, touch_detector::TouchDetector, ColorResolution, Counter,
    Format, KeyboardEvent, Resolution, DEFAULT_PIXEL_COUNT, DEFAULT_RESOLUTION,
};

#[show_image::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    //
    // Looking for a device. If a device has been found, it will be opened and a stream started.
    let mut device = match Device::init() {
        Ok(d) => d,
        Err(msg) => {
            println!("{}", msg);
            return Ok(());
        }
    };

    // Choosing the depth measuring range for DCAM560 (Near, Mid, or Far)
    #[cfg(feature = "dcam560")]
    {
        set_depth_measuring_range_dcam560(&device, DepthRange::Mid);

        let range = get_depth_measuring_range(&device);
        println!("depth measuring range: {} mm to {} mm", range.0, range.1);
    }

    // Choosing the min/max depth in mm for the color mapping of the depth output. These values also bound the depths used in the `TochDetector` to reduce measuring artifacts. In the specs the depth measuring range for the NYX650 is given as min: 300 mm, max: 4500 mm. The depth measuring range for the DCAM560 depends on the range chosen above.
    device.set_depth_range(500, 1000);

    let mut touch_detector = TouchDetector::new(&device, 5.0, 50.0, 30, 5, DEFAULT_PIXEL_COUNT);

    // Setting the color resolution. If not set the default will be 640x480.
    // If mapper is set to true, resolution setting will be ignored and reverted to 640x480.
    device.set_color_resolution(ColorResolution::Res640x480);

    // The normal color frame Type. Might be reset below if mapped.
    let mut color_frame_type = FrameType::Color;

    // If map_color is set to true color_resolution is reset to 640x480.
    let map_color = true;
    if map_color {
        color_frame_type = FrameType::ColorMapped;
        device.map_color_to_depth(map_color);
    }

    let color_resolution = device.get_color_resolution();

    // Choose between RGB and BGR color format, default is BGR.
    device.set_color_format(ColorFormat::Rgb);

    // vectors to store image data
    let mut signal = new_fixed_vec(DEFAULT_PIXEL_COUNT, 0u8); // 8 bit per pixel
    let mut distance = new_fixed_vec(DEFAULT_PIXEL_COUNT, 0.0f32); // 32 bit per pixel
    let mut rgb = new_fixed_vec(3 * DEFAULT_PIXEL_COUNT, 0u8); // 24 bit per pixel

    // creating the image windows, `double()` doubles the size of the window in both dimensions
    let depth_window = create_window("depth", &DEFAULT_RESOLUTION.double(), true);
    let touch_window = create_window("touch", &DEFAULT_RESOLUTION.double(), true);
    let color_window = create_window("color", &color_resolution.double(), true);

    let stop = KeyboardEvent::new("\n");

    let mut counter = Counter::new(10);

    let mut init = true;

    ///////////////////////////////////////////////////////////////////////////
    // main loop reading frames and displaying them
    loop {
        // Scepter API has an additional `max_wait_time_ms` paramter
        #[cfg(not(feature = "dcam560"))]
        read_next_frame(&mut device, 1200);

        #[cfg(feature = "dcam560")]
        read_next_frame(&device, frame_ready);

        // depth ______________________________________________________________

        get_frame(&mut device, &FrameType::Depth, &mut signal);
        // touch_detector.get_normalized_average_depth(signal);

        // apply Google's Turbo color map
        for (i, si) in signal.iter().enumerate() {
            rgb[3 * i..3 * i + 3].copy_from_slice(&TURBO[*si as usize]);
        }

        update_window(&depth_window, &DEFAULT_RESOLUTION, &rgb, Format::Rgb);

        // touch detector _____________________________________________________

        // the last get_frame() call before this should have FrameType::Depth
        touch_detector.process(&device, &mut signal, &mut distance);

        update_window(&touch_window, &DEFAULT_RESOLUTION, &signal, Format::Mono);

        // color ________________________________________________________________

        get_frame(&mut device, &color_frame_type, &mut rgb);

        if init {
            init = false;
            device.check_pixel_count(rgb.len() / 3);
            println!("frame info: {:?}", device.get_frame_info());
            println!("press Enter to quit")
        }

        update_window(&color_window, &color_resolution, &rgb, Format::Rgb);

        //_____________________________________________________________________

        counter.print_fps_frame_count_info();

        if stop.key_was_pressed() {
            break;
        }
    }

    stop.join();

    device.shut_down();

    Ok(())
}

// helper functions using the show_image crate ////////////////////////////////

use show_image::{ImageInfo, ImageView, WindowOptions, WindowProxy};

fn update_window(window: &WindowProxy, resolution: &Resolution, data: &[u8], format: Format) {
    let (w, h) = resolution.to_tuple();
    let info = match format {
        Format::Mono => ImageInfo::mono8(w, h),
        Format::Rgb => ImageInfo::rgb8(w, h),
        Format::Bgr => ImageInfo::bgr8(w, h),
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
