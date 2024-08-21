use crate::tests;
use struct_iterable::Iterable;


/// Specification for SPI signal timings for bit banging.
struct SignalTimingRequirementsInNs {
    /// Minimum limit (signal cannot be shorter than this).
    minimum: u32,
    /// Signal time we should be aiming for.
    typical: u32,
    /// Maximum limit (signal cannot be longer than this).
    maximum: u32,
}

/// Precise timings necessary to drive a WS2812B LED strip.
#[derive(Iterable)]
pub struct WS2812BRequirements {
    /// How much time signal should be HIGH to transmit a bit of value 0.
    zero_code_high_voltage_time: SignalTimingRequirementsInNs,
    /// How much time signal should be LOW to transmit a bit of value 0.
    zero_code_low_voltage_time: SignalTimingRequirementsInNs,
    /// How much time signal should be HIGH to transmit a bit of value 1.
    one_code_high_voltage_time: SignalTimingRequirementsInNs,
    /// How much time signal should be LOW to transmit a bit of value 1.
    one_code_low_voltage_time: SignalTimingRequirementsInNs,
    /// How much time signal should be LOW to indicate all the data has been sent.
    latch_low_voltage_time: SignalTimingRequirementsInNs,
}

/// Data from [josh.com](https://wp.josh.com/2014/05/13/ws2812-neopixels-are-not-so-finicky-once-you-get-to-know-them/).
pub const DEFAULT_WS2812B_TIMING_REQUIREMENTS: WS2812BRequirements = WS2812BRequirements {
    zero_code_high_voltage_time: SignalTimingRequirementsInNs { minimum: 200, typical: 350, maximum: 500 },
    zero_code_low_voltage_time: SignalTimingRequirementsInNs { minimum: 450, typical: 600, maximum: 5_000 },
    one_code_high_voltage_time: SignalTimingRequirementsInNs { minimum: 550, typical: 700, maximum: 5_500 },
    one_code_low_voltage_time: SignalTimingRequirementsInNs { minimum: 450, typical: 600, maximum: 5_000 },
    latch_low_voltage_time: SignalTimingRequirementsInNs { minimum: 250_000, typical: 250_500, maximum: 251_000 },
    // for old models:
    // latch_low_voltage_time: SignalTimingRequirementsInNs { minimum: 6_000, typical: 6_500, maximum: 10_000 },
};

/// Translates frequency (Hertz) to how many nanoseconds one cycle (one bit) will take.
pub fn get_nanos_per_cycle(clock_speed_in_hz: u32) -> u32 {
    1_000_000_000 / clock_speed_in_hz
}

tests! {
    get_nanos_per_cycle_tests,

    |(input, expected): (u32, u32)| {
        let actual = get_nanos_per_cycle(input);
        assert_eq!(expected, actual);
    },

    given_1GHz_should_return_1ns: (1_000_000_000, 1),
    given_1MHz_should_return_1000ns: (1_000_000, 1_000),
    given_1kHz_should_return_1000000ns: (1_000, 1_000_000),
    given_1Hz_should_return_1000000000ns: (1, 1_000_000_000),
    given_8MHz_should_return_125ns: (8_000_000, 125),
}

pub fn clock_speed_matches_requirements(clock_speed_in_hz: u32, requirements: WS2812BRequirements) -> bool {
    for (signal_name, signal_requirements_as_any) in requirements.iter() {
        let signal_requirements = match signal_requirements_as_any.downcast_ref::<SignalTimingRequirementsInNs>() {
            Some(signal_requirements) => signal_requirements,
            None => panic!("field {} is not an instance of SignalTimingRequirementsInNs!", signal_name),
        };

        if get_nanos_per_cycle(clock_speed_in_hz) >= signal_requirements.maximum {
            return false;
        }
    }
    true
}

tests! {
    clock_speed_matches_requirements_tests,

    |(input, expected): (u32, bool)| {
        let actual = clock_speed_matches_requirements(input, DEFAULT_WS2812B_TIMING_REQUIREMENTS);
        assert_eq!(expected, actual);
    },

    given_1GHz_should_return_true: (1_000_000_000, true),
    given_1MHz_should_return_false: (1_000_000, false),
    given_1kHz_should_return_false: (1_000, false),
    given_1Hz_should_return_false: (1, false),
    given_8MHz_should_return_true: (8_000_000, true),
}

pub struct WS2812BSpecification {
    pub zero_code: Vec<u8>,
    pub one_code: Vec<u8>,
    pub latch: Vec<u8>,
}

pub fn get_signal_representation_in_bytes(clock_speed_in_hz: u32, requirements: WS2812BRequirements) -> WS2812BSpecification {
    // TODO: actually figure this out
    //         let minimumCycles = signal_requirements.minimum / nanos_per_cycle;
    //         let typicalCycles = signal_requirements.typical / nanos_per_cycle;
    //         let maximumCycles = signal_requirements.maximum / nanos_per_cycle;
    //         println!("cycles for {}Hz and {}", clock_speed_in_hz, signal_name);
    //         println!("\tminimum = {}", minimumCycles);
    //         println!("\ttypical = {}", typicalCycles);
    //         println!("\tmaximum = {}", maximumCycles);
    WS2812BSpecification {
        zero_code: vec![0b11100000, 0b00000000],
        one_code: vec![0b11111100, 0b00000000],
        latch: vec![0; 251],
    }
}

tests! {
    get_signal_representation_in_bytes_tests,

    |(input, expected): (u32, WS2812BSignalBytes)| {
        let actual = get_signal_representation_in_bytes(input, DEFAULT_WS2812B_TIMING_REQUIREMENTS);
        assert_eq!(*expected, *actual);
    },
}

