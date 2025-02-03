//! The latest Scepter API for NYX650/660, DS86/87, DS77C, and DS77 cameras. See the [repository](https://github.com/ScepterSW/ScepterSDK) and the [API docs](https://github.com/ScepterSW/Scepter-Wiki/blob/master/en/ScepterSDK/BaseSDK.md).

pub mod device;
pub mod frame;

/// Status messages from numeric code.
const fn get_message(code: i32) -> &'static str {
    match code {
        0 => "OK",                              // The function completed successfully.
        -1 => "DEVICE_IS_LIMBO",                // The device is limbo
        -2 => "INVALID_DEVICE_INDEX",           // The input device index is invalid.
        -3 => "DEVICE_POINTER_IS_NULL",         // The device structure pointer is null.
        -4 => "INVALID_FRAME_TYPE",             // The input frame type is invalid.
        -5 => "FRAME_POINTER_IS_NULL",          // The output frame buffer is null.
        -6 => "NO_PROPERTY_VALUE_GET",          // Cannot get the value for the specified property.
        -7 => "NO_PROPERTY_VALUE_SET",          // Cannot set the value for the specified property.
        -8 => "PROPERTY_POINTER_IS_NULL",       // The input property value buffer pointer is null.
        -9 => "PROPERTY_SIZE_NOT_ENOUGH", // The input property value buffer size is too small to store the specified property value.
        -10 => "INVALID_DEPTH_RANGE",     // The input depth range mode is invalid.
        -11 => "GET_FRAME_READY_TIME_OUT", // Capture the next image frame time out.
        -12 => "INPUT_POINTER_IS_NULL",   // An input pointer parameter is null.
        -13 => "CAMERA_NOT_OPENED",       // The camera has not been opened.
        -14 => "INVALID_CAMERA_TYPE",     // The specified type of camera is invalid.
        -15 => "INVALID_PARAMS", // One or more of the parameter values provided are invalid.
        -16 => "CURRENT_VERSION_NOT_SUPPORT", // This feature is not supported in the current version.
        -17 => "UPGRADE_IMG_ERROR",           // There is an error in the upgrade file.
        -18 => "UPGRADE_IMG_PATH_TOO_LONG",   // Upgrade file path length greater than 260.
        -19 => "UPGRADE_CALLBACK_NOT_SET",    // scSetUpgradeSTATUSCallback is not called.
        -20 => "PRODUCT_NOT_SUPPORT", // The current product does not support this operation.
        -21 => "NO_CONFIG_FOLDER",    // No product profile found.
        -22 => "WEB_SERVER_START_ERROR", // WebServer Start/Restart error(IP or PORT 8080).
        -23 => "GET_OVER_STAY_FRAME", // The time from frame ready to get frame is out of 1s.
        -24 => "CREATE_LOG_DIR_ERROR", // Create log directory error.
        -25 => "CREATE_LOG_FILE_ERROR", // Create log file error.
        -100 => "NO_ADAPTER_CONNECTED", // There is no adapter connected.
        -101 => "REINITIALIZED",      // The SDK has been Initialized.
        -102 => "NO_INITIALIZED",     // The SDK has not been Initialized.
        -103 => "CAMERA_OPENED",      // The camera has been opened.
        -104 => "CMD_ERROR",          // Set/Get cmd control error.
        -105 => "CMD_SYNC_TIME_OUT",  // Set cmd ok.but time out for the sync return.
        -106 => "IP_NOT_MATCH",       // IP is not in the same network segment.
        -107 => "NOT_STOP_STREAM",    // Please invoke scStopStream first to close the data stream.
        -108 => "NOT_START_STREAM",   // Please invoke scStartStream first to get the data stream.
        -109 => "NOT_FIND_DRIVERS_FOLDER", // Please check whether the Drivers directory exists.
        -110 => "CAMERA_OPENING",     // The camera is openin,by another Sc_OpenDeviceByXXX API.
        -111 => "CAMERA_OPENED_BY_ANOTHER_APP", // The camera has been opened by another APP.
        -112 => "GET_AI_RESULT_TIME_OUT", // Capture the next AI result time out.
        -113 => "MORPH_AI_LIB_ERROR", // The morph Al library is not exist or initialized failed.
        -114 => "CPU_AFFINITY_CHECK_FAILED", // The cpu affinity config file check failed
        -255 => "OTHERS",             // An unknown error occurred.
        _ => "_",
    }
}
