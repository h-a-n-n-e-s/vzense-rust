//! The latest Scepter API for NYX650/660, DS86/87, DS77C, and DS77 cameras. See the [repository](https://github.com/ScepterSW/ScepterSDK) and the [API docs](https://github.com/ScepterSW/Scepter-Wiki/blob/master/en/ScepterSDK/BaseSDK.md).

pub mod device;
pub mod frame;

/// Status codes.
pub fn get_status_codes() -> [String; 255] {
    let mut status = [const { String::new() }; 255];

    status[0] = "OK".to_string(); // The function completed successfully.
    status[1] = "DEVICE_IS_LIMBO".to_string(); // The device is limbo
    status[2] = "INVALID_DEVICE_INDEX".to_string(); // The input device index is invalid.
    status[3] = "DEVICE_POINTER_IS_NULL".to_string(); // The device structure pointer is null.
    status[4] = "INVALID_FRAME_TYPE".to_string(); // The input frame type is invalid.
    status[5] = "FRAME_POINTER_IS_NULL".to_string(); // The output frame buffer is null.
    status[6] = "NO_PROPERTY_VALUE_GET".to_string(); // Cannot get the value for the specified property.
    status[7] = "NO_PROPERTY_VALUE_SET".to_string(); // Cannot set the value for the specified property.
    status[8] = "PROPERTY_POINTER_IS_NULL".to_string(); // The input property value buffer pointer is null.
    status[9] = "PROPERTY_SIZE_NOT_ENOUGH".to_string(); // The input property value buffer size is too small to store the specified property value.
    status[10] = "INVALID_DEPTH_RANGE".to_string(); // The input depth range mode is invalid.
    status[11] = "GET_FRAME_READY_TIME_OUT".to_string(); // Capture the next image frame time out.
    status[12] = "INPUT_POINTER_IS_NULL".to_string(); // An input pointer parameter is null.
    status[13] = "CAMERA_NOT_OPENED".to_string(); // The camera has not been opened.
    status[14] = "INVALID_CAMERA_TYPE".to_string(); // The specified type of camera is invalid.
    status[15] = "INVALID_PARAMS".to_string(); // One or more of the parameter values provided are invalid.
    status[16] = "CURRENT_VERSION_NOT_SUPPORT".to_string(); // This feature is not supported in the current version.
    status[17] = "UPGRADE_IMG_ERROR".to_string(); // There is an error in the upgrade file.
    status[18] = "UPGRADE_IMG_PATH_TOO_LONG".to_string(); // Upgrade file path length greater than 260.
    status[19] = "UPGRADE_CALLBACK_NOT_SET".to_string(); // scSetUpgradeStatusCallback is not called.
    status[20] = "PRODUCT_NOT_SUPPORT".to_string(); // The current product does not support this operation.
    status[21] = "NO_CONFIG_FOLDER".to_string(); // No product profile found.
    status[22] = "WEB_SERVER_START_ERROR".to_string(); // WebServer Start/Restart error(IP or PORT 8080).
    status[23] = "GET_OVER_STAY_FRAME".to_string(); // The time from frame ready to get frame is out of 1s.
    status[24] = "CREATE_LOG_DIR_ERROR".to_string(); // Create log directory error.
    status[25] = "CREATE_LOG_FILE_ERROR".to_string(); // Create log file error.
    status[100] = "NO_ADAPTER_CONNECTED".to_string(); // There is no adapter connected.
    status[101] = "REINITIALIZED".to_string(); // The SDK has been Initialized.
    status[102] = "NO_INITIALIZED".to_string(); // The SDK has not been Initialized.
    status[103] = "CAMERA_OPENED".to_string(); // The camera has been opened.
    status[104] = "CMD_ERROR".to_string(); // Set/Get cmd control error.
    status[105] = "CMD_SYNC_TIME_OUT".to_string(); // Set cmd ok.but time out for the sync return.
    status[106] = "IP_NOT_MATCH".to_string(); // IP is not in the same network segment.
    status[107] = "NOT_STOP_STREAM".to_string(); // Please invoke scStopStream first to close the data stream.
    status[108] = "NOT_START_STREAM".to_string(); // Please invoke scStartStream first to get the data stream.
    status[109] = "NOT_FIND_DRIVERS_FOLDER".to_string(); // Please check whether the Drivers directory exists.
    status[110] = "CAMERA_OPENING".to_string(); // The camera is openin,by another Sc_OpenDeviceByXXX API.
    status[111] = "CAMERA_OPENED_BY_ANOTHER_APP".to_string(); // The camera has been opened by another APP.
    status[112] = "GET_AI_RESULT_TIME_OUT".to_string(); // Capture the next AI result time out.
    status[113] = "MORPH_AI_LIB_ERROR".to_string(); // The morph Al library is not exist or initialized failed.
    status[114] = "CPU_AFFINITY_CHECK_FAILED".to_string(); // The cpu affinity config file check failed
    status[255] = "OTHERS".to_string(); // An unknown error occurred.

    status
}
