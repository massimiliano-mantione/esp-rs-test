use embedded_svc::{ipv4, wifi::*};
use esp_idf_hal::peripheral;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_svc::ping;
use esp_idf_svc::{eventloop::EspSystemEventLoop, wifi::*};
use esp_idf_sys::{
    // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
    self as _,
    camera_config_t,
    camera_config_t__bindgen_ty_1,
    camera_config_t__bindgen_ty_2,
    camera_fb_location_t_CAMERA_FB_IN_PSRAM,
    camera_grab_mode_t_CAMERA_GRAB_LATEST,
    esp_camera_deinit,
    esp_camera_fb_get,
    esp_camera_fb_return,
    esp_camera_init,
    esp_camera_sensor_get,
    framesize_t_FRAMESIZE_QQVGA,
    ledc_channel_t_LEDC_CHANNEL_0,
    ledc_timer_t_LEDC_TIMER_0,
    pixformat_t_PIXFORMAT_JPEG,
    timeval,
    EspError,
    ESP_OK,
};
use log::*;

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

const XCLK_FREQ_HZ: i32 = 20000000;

#[allow(dead_code)]
const SSID: &str = env!("RUST_ESP32_STD_DEMO_WIFI_SSID");
#[allow(dead_code)]
const PASS: &str = env!("RUST_ESP32_STD_DEMO_WIFI_PASS");

fn timeval_usec(t: timeval) -> u64 {
    (t.tv_sec as u64 * 1000000) + (t.tv_usec as u64)
}

fn ping_address(ip: ipv4::Ipv4Addr) -> Result<(), EspError> {
    info!("About to do some pings for {:?}", ip);

    let ping_summary = ping::EspPing::default().ping(ip, &Default::default())?;
    if ping_summary.transmitted != ping_summary.received {
        error!("Pinging IP {} resulted in timeouts", ip);
    }

    info!("Pinging done");

    Ok(())
}

fn connect_wifi(
    modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
) -> Result<Box<EspWifi<'static>>, EspError> {
    let mut esp_wifi = EspWifi::new(modem, sysloop.clone(), None)?;

    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))?;

    info!("Starting wifi...");

    wifi.start()?;

    info!("Scanning...");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == SSID);

    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            SSID, ours.channel
        );
        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            SSID
        );
        None
    };

    wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration {
            ssid: SSID.into(),
            password: PASS.into(),
            channel,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "aptest".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    ))?;

    info!("Connecting wifi...");

    wifi.connect()?;

    info!("Waiting for DHCP lease...");

    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    info!("Wifi DHCP info: {:?}", ip_info);

    ping_address(ip_info.subnet.gateway)?;

    Ok(Box::new(esp_wifi))
}

fn main() -> Result<(), EspError> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    #[allow(unused)]
    let peripherals = Peripherals::take().unwrap();
    #[allow(unused)]
    let pins = peripherals.pins;
    #[allow(unused)]
    let sysloop = EspSystemEventLoop::take().unwrap();

    let wifi_connection = connect_wifi(peripherals.modem, sysloop.clone())?;

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
        xclk_freq_hz: XCLK_FREQ_HZ,
        ledc_timer: ledc_timer_t_LEDC_TIMER_0,
        ledc_channel: ledc_channel_t_LEDC_CHANNEL_0,
        pixel_format: pixformat_t_PIXFORMAT_JPEG,
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
            return Ok(());
        }

        let _s = esp_camera_sensor_get();

        let mut previous_t = 0;
        let mut skipped = 0;
        let mut reported_t = 0;
        let mut reported_frames = 0;
        let mut fb_len_sum = 0;
        let mut fb_len_max = 0;
        const REPORT_DT_USEC: u64 = 5000000;

        for _ in 0..1000 {
            let fb = esp_camera_fb_get();
            match fb.as_ref() {
                Some(fb) => {
                    let t = timeval_usec(fb.timestamp);
                    reported_frames += 1;

                    if t != previous_t {
                        previous_t = t;
                        fb_len_sum += fb.len;
                        fb_len_max = fb_len_max.max(fb.len);

                        let dt = t - reported_t;
                        if dt >= REPORT_DT_USEC {
                            reported_t = t;

                            let (frame_dt_avg, fb_len_avg, fr) = if dt > 0 {
                                (
                                    dt / reported_frames,
                                    fb_len_sum / (reported_frames as usize),
                                    1000000.0 / ((dt / reported_frames) as f32),
                                )
                            } else {
                                (0, 0, 0.0)
                            };
                            println!(
                                "skipped {} count {} dt {} (fr {}, len avg {} max {})",
                                skipped, reported_frames, frame_dt_avg, fr, fb_len_avg, fb_len_max
                            );
                            reported_frames = 0;
                            fb_len_sum = 0;
                            fb_len_max = 0;

                            skipped = 0;
                        }
                    } else {
                        skipped += 1;
                    }
                }
                None => {
                    println!("esp_camera_fb_get failed");
                    return Ok(());
                }
            }
            esp_camera_fb_return(fb);
        }

        esp_camera_deinit();
    }

    drop(wifi_connection);
    println!("Done.");
    Ok(())
}
