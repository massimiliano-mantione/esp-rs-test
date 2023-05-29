use esp_idf_sys::{
    self as _, camera_config_t__bindgen_ty_1, camera_config_t__bindgen_ty_2,
    camera_fb_location_t_CAMERA_FB_IN_PSRAM, camera_grab_mode_t_CAMERA_GRAB_LATEST,
    framesize_t_FRAMESIZE_QQVGA, ledc_channel_t_LEDC_CHANNEL_0, ledc_timer_t_LEDC_TIMER_0,
    pixformat_t_PIXFORMAT_YUV422, timeval, ESP_OK,
}; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use esp_idf_sys::{
    camera_config_t, esp_camera_deinit, esp_camera_fb_get, esp_camera_fb_return, esp_camera_init,
    esp_camera_sensor_get,
};

const CAMERA_PWDN_GPIO_NUM: i32 = -1;
const CAMERA_RESET_GPIO_NUM: i32 = -1;
const CAMERA_XCLK_GPIO_NUM: i32 = 15;
const CAMERA_SIOD_GPIO_NUM: i32 = 4;
const CAMERA_SIOC_GPIO_NUM: i32 = 5;

const CAMERA_Y9_GPIO_NUM: i32 = 16;
const CAMERA_Y8_GPIO_NUM: i32 = 17;
const CAMERA_Y7_GPIO_NUM: i32 = 18;
const CAMERA_Y6_GPIO_NUM: i32 = 12;
const CAMERA_Y5_GPIO_NUM: i32 = 10;
const CAMERA_Y4_GPIO_NUM: i32 = 8;
const CAMERA_Y3_GPIO_NUM: i32 = 9;
const CAMERA_Y2_GPIO_NUM: i32 = 11;
const CAMERA_VSYNC_GPIO_NUM: i32 = 6;
const CAMERA_HREF_GPIO_NUM: i32 = 7;
const CAMERA_PCLK_GPIO_NUM: i32 = 13;

fn timeval_usec(t: timeval) -> u64 {
    (t.tv_sec as u64 * 1000000) + (t.tv_usec as u64)
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    println!("Setting up camera");

    let camera_config = camera_config_t {
        pin_pwdn: CAMERA_PWDN_GPIO_NUM,
        pin_reset: CAMERA_RESET_GPIO_NUM,
        pin_xclk: CAMERA_XCLK_GPIO_NUM,
        pin_d7: CAMERA_Y9_GPIO_NUM,
        pin_d6: CAMERA_Y8_GPIO_NUM,
        pin_d5: CAMERA_Y7_GPIO_NUM,
        pin_d4: CAMERA_Y6_GPIO_NUM,
        pin_d3: CAMERA_Y5_GPIO_NUM,
        pin_d2: CAMERA_Y4_GPIO_NUM,
        pin_d1: CAMERA_Y3_GPIO_NUM,
        pin_d0: CAMERA_Y2_GPIO_NUM,
        pin_vsync: CAMERA_VSYNC_GPIO_NUM,
        pin_href: CAMERA_HREF_GPIO_NUM,
        pin_pclk: CAMERA_PCLK_GPIO_NUM,
        xclk_freq_hz: 20000000,
        ledc_timer: ledc_timer_t_LEDC_TIMER_0,
        ledc_channel: ledc_channel_t_LEDC_CHANNEL_0,
        pixel_format: pixformat_t_PIXFORMAT_YUV422,
        frame_size: framesize_t_FRAMESIZE_QQVGA,
        jpeg_quality: 12,
        fb_count: 3,
        fb_location: camera_fb_location_t_CAMERA_FB_IN_PSRAM,
        grab_mode: camera_grab_mode_t_CAMERA_GRAB_LATEST,
        sccb_i2c_port: Default::default(),
        __bindgen_anon_1: camera_config_t__bindgen_ty_1 {
            pin_sscb_sda: CAMERA_SIOD_GPIO_NUM,
        },
        __bindgen_anon_2: camera_config_t__bindgen_ty_2 {
            pin_sscb_scl: CAMERA_SIOC_GPIO_NUM,
        },
    };

    unsafe {
        let err = esp_camera_init(&camera_config);
        if err != ESP_OK {
            println!("esp_camera_init {}", err);
            return;
        }

        let _s = esp_camera_sensor_get();

        let mut previous_t = 0;
        let mut skipped = 0;

        for _ in 0..1000 {
            let fb = esp_camera_fb_get();
            match fb.as_ref() {
                Some(fb) => {
                    let t = timeval_usec(fb.timestamp);

                    if t != previous_t {
                        let dt = t - previous_t;
                        previous_t = t;
                        println!(
                            "fb w {} h {} len {} dt {} (skipped {})",
                            fb.width, fb.height, fb.len, dt, skipped
                        );
                        skipped = 0;
                    } else {
                        skipped += 1;
                    }
                }
                None => {
                    println!("esp_camera_fb_get failed");
                    return;
                }
            }
            esp_camera_fb_return(fb);
        }

        esp_camera_deinit();
    }

    println!("Done.");
}
