/*!
This example covers all the functionality provided by the library. It connects to a device, starts a stream, and displays the received data. The `touch_detector` is a very simple example of image processing, using depth data to detect touch events.
*/

// By default using the newest Scepter API.
#[cfg(not(feature = "dcam560"))]
use vzense_rust::scepter as camera_api;

// Uses an older API specifically for the DCAM560 model.
#[cfg(feature = "dcam560")]
use vzense_rust::dcam560 as camera_api;

use camera_api::{
    device::Device,
    frame::{get_color_frame, get_depth_scaled_u8_frame, read_next_frame},
};

use vzense_rust::{
    util::{
        color_map::TURBO, new_fixed_vec, touch_detector::TouchDetector, Counter, KeyboardEvent,
    },
    ColorFormat, ColorResolution, Resolution, DEFAULT_PIXEL_COUNT, DEFAULT_RESOLUTION,
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

    // Choose the depth measuring range for DCAM560 (Near, Mid, or Far).
    #[cfg(feature = "dcam560")]
    {
        device.set_depth_measuring_range(vzense_rust::DepthMeasuringRange::Near);

        let range = device.get_depth_measuring_range();
        println!("depth measuring range: {} mm to {} mm", range.0, range.1);
    }

    // Choose between RGB and BGR color format, default is BGR.
    device.set_color_format(ColorFormat::Rgb);

    // Choose the min/max depth in mm for the color mapping of the depth output. These values also bound the depths used in the `TochDetector` to reduce measuring artifacts. In the specs the depth measuring range for the NYX650 is given as min: 300 mm, max: 4500 mm. The depth measuring range for the DCAM560 depends on the range chosen above.
    device.set_depth_range(160, 1100);

    // Initialize the touch detector.
    let mut touch_detector = TouchDetector::new(&device, 5.0, 50.0, 30, 5, DEFAULT_PIXEL_COUNT);

    // Mapping color to depth frame. If set to true, the color_resolution is fixed to 640x480.
    device.map_color_to_depth(false);

    // Setting the color resolution. If not set the default will be 640x480.
    // If color is mapped to depth, color resolution setting will be ignored.
    let color_resolution = device.set_color_resolution(ColorResolution::Res800x600);

    // Vectors to store image data.

    // let mut ir = new_fixed_vec(DEFAULT_PIXEL_COUNT, 0u8); // 8 bit per pixel
    // let mut depth_mm = new_fixed_vec(DEFAULT_PIXEL_COUNT, 0u16); // 16 bit per pixel
    let mut depth_scaled = new_fixed_vec(DEFAULT_PIXEL_COUNT, 0u8); // 8 bit per pixel
    let mut touch = new_fixed_vec(DEFAULT_PIXEL_COUNT, 0u8); // 8 bit per pixel
    let mut distance = new_fixed_vec(DEFAULT_PIXEL_COUNT, 0.0f32); // 32 bit per pixel

    let mut depth_rgb = new_fixed_vec(3 * DEFAULT_PIXEL_COUNT, 0u8); // 24 bit per pixel
    let mut color_rgb = new_fixed_vec(3 * color_resolution.to_pixel_count(), 0u8); // 24 bit per pixel

    // Creating the image windows, `double()` doubles the size of the window in both dimensions.

    // let ir_window = create_window("ir", &DEFAULT_RESOLUTION.double(), true);
    let depth_window = create_window("depth", &DEFAULT_RESOLUTION.double(), true);
    let touch_window = create_window("touch", &DEFAULT_RESOLUTION.double(), true);
    let color_window = create_window("color", &color_resolution.double(), true);

    let stop = KeyboardEvent::new("\n");

    // For fps and frame count output.
    let mut counter = Counter::new(10);

    let mut init = true;

    ///////////////////////////////////////////////////////////////////////////
    // main loop reading frames and displaying them
    loop {
        // `read_next_frame()` must be called at the beginning of each loop to retrieve new data.

        // Scepter API has an additional `max_wait_time_ms` paramter.
        #[cfg(not(feature = "dcam560"))]
        read_next_frame(&mut device, 100);

        #[cfg(feature = "dcam560")]
        read_next_frame(&mut device);

        // IR (only for Scepter API) __________________________________________

        // get_ir_frame(&mut device, &mut ir);

        // update_window(&ir_window, &DEFAULT_RESOLUTION, &ir, Format::Mono);

        // depth ______________________________________________________________

        // raw depth data in mm
        // get_depth_mm_u16_frame(&mut device, &mut depth_mm);

        // scaled depth data
        get_depth_scaled_u8_frame(&mut device, &mut depth_scaled);

        // apply color map
        for (i, dsi) in depth_scaled.iter().enumerate() {
            depth_rgb[3 * i..3 * i + 3].copy_from_slice(&TURBO[*dsi as usize]);
        }

        update_window(&depth_window, &DEFAULT_RESOLUTION, &depth_rgb, Format::Rgb);

        // touch detector
        // should be called after get_depth... call, otherwise `process` does nothing.
        touch_detector.process(&device, &mut touch, &mut distance);

        update_window(&touch_window, &DEFAULT_RESOLUTION, &touch, Format::Mono);

        // color ________________________________________________________________

        get_color_frame(&mut device, &mut color_rgb);

        if init {
            init = false;
            device.check_pixel_count(color_rgb.len() / 3);
            println!("frame info: {}", device.get_frame_info());
            println!("press Enter to quit")
        }

        update_window(&color_window, &color_resolution, &color_rgb, Format::Rgb);

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

// helper functions using the show_image crate ________________________________

/// Image formats.
pub enum Format {
    Mono,
    Rgb,
    Bgr,
}

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
