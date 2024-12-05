mod strip;
use strip::{create_ws2812b_strip, LedStripController};
use jni::objects::JClass;
use jni::sys::jint;
use jni::JNIEnv;
use std::sync::{Mutex, OnceLock};
/*
// TODO: oh my GOD RUST SUCKS WITH GLOBALS (compile to see why)
fn strip(leds_count: usize) -> &'static Mutex<dyn LedStripController> {
    static STRIP: OnceLock<Mutex<dyn LedStripController>> = OnceLock::new();
    STRIP.get_or_init(|| Mutex::new(create_ws2812b_strip(leds_count)))
}
*/
#[no_mangle]
pub extern "system" fn Java_jezor_jni_RPi5RP1SPI_initializeStrip<'local>(
    mut env: JNIEnv<'local>,
    class: JClass<'local>,
    leds_count: jint,
) {
    println!("initialize called");
    //strip(leds_count as usize);
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
    println!("set led called");
    // strip(0)
    //     .get()
    //     .lock()
    //     .unwrap()
    //     .set(led_index as usize, red as u8, green as u8, blue as u8);
}

#[no_mangle]
pub extern "system" fn Java_jezor_jni_RPi5RP1SPI_renderStrip<'local>(
    mut env: JNIEnv<'local>,
    class: JClass<'local>,
) {
    println!("render called");
    // strip(0)
    //     .get()
    //     .lock()
    //     .unwrap()
    //     .commit()
}

#[no_mangle]
pub extern "system" fn Java_jezor_jni_RPi5RP1SPI_closeStrip<'local>(
    mut env: JNIEnv<'local>,
    class: JClass<'local>,
) {
    println!("close called");
}
