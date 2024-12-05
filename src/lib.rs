mod strip;
use jni::objects::JClass;
use jni::sys::jint;
use jni::JNIEnv;
use std::sync::{Mutex, OnceLock};
use strip::create_ws2812b_strip;
use strip::LedStripController;
use strip::SpiLedStripController;

fn strip(leds_count: usize) -> &'static Mutex<SpiLedStripController> {
    static STRIP: OnceLock<Mutex<SpiLedStripController>> = OnceLock::new();
    STRIP.get_or_init(|| Mutex::new(create_ws2812b_strip(leds_count).unwrap()))
}
#[no_mangle]
pub extern "system" fn Java_jezor_jni_RPi5RP1SPI_initializeStrip<'local>(
    mut env: JNIEnv<'local>,
    class: JClass<'local>,
    leds_count: jint,
) {
    strip(leds_count as usize);
}

#[no_mangle]
pub extern "system" fn Java_jezor_jni_RPi5RP1SPI_setLed<'local>(
    mut env: JNIEnv<'local>,
    class: JClass<'local>,
    led_index: jint,
    red: jint,
    green: jint,
    blue: jint,
) {
    strip(0)
        .lock()
        .unwrap()
        .set(led_index as usize, red as u8, green as u8, blue as u8);
}

#[no_mangle]
pub extern "system" fn Java_jezor_jni_RPi5RP1SPI_renderStrip<'local>(
    mut env: JNIEnv<'local>,
    class: JClass<'local>,
) {
    strip(0).lock().unwrap().commit().unwrap();
}

#[no_mangle]
pub extern "system" fn Java_jezor_jni_RPi5RP1SPI_closeStrip<'local>(
    mut env: JNIEnv<'local>,
    class: JClass<'local>,
) {
    println!("close called");
}
