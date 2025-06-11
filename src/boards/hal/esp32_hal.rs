// ESP32 HAL implementation using esp-idf-hal
use crate::boards::{HalPinState, BoardError, PlatformHal};
#[cfg(feature = "esp32")]
use esp_idf_hal::gpio::{GpioPin, Output, Input, PinDriver};
#[cfg(feature = "esp32")]
use esp_idf_hal::ledc::{LedcDriver, LedcTimerDriver, LedcChannel, LedcTimer, LedcTimerConfig, LedcTimerSpeed, LedcTimerBit, LedcResolution, LedcChannelDriver};

pub struct EspIdfHal {
    // Optionally store I2C/SPI handles if needed
}

impl EspIdfHal {
    pub fn new() -> Result<Self, BoardError> {
        Ok(EspIdfHal {})
    }
    /// Get a mutable reference to the I2C bus (for ADS1115)
    #[cfg(feature = "esp32")]
    pub fn i2c(&mut self) -> Option<&mut esp_idf_hal::i2c::I2cDriver<'static>> {
        // You would store and return a reference to an I2cDriver if you add it to the struct
        None // Not implemented in this scaffold
    }
    /// Get a mutable reference to the SPI bus (for ILI9341)
    #[cfg(feature = "esp32")]
    pub fn spi(&mut self) -> Option<&mut esp_idf_hal::spi::SpiDriver<'static>> {
        // You would store and return a reference to a SpiDriver if you add it to the struct
        None // Not implemented in this scaffold
    }
}

impl PlatformHal for EspIdfHal {
    type OutputPin = PinDriver<'static, Output>;
    type InputPin = PinDriver<'static, Input>;
    type PwmPin = LedcChannelDriver<'static>;
    type PinError = esp_idf_hal::gpio::Error;

    fn setup_pin_output(&self, gpio_pin_num: u8) -> Result<Self::OutputPin, BoardError> {
        let pin = unsafe { GpioPin::new(gpio_pin_num).map_err(|e| BoardError::HalError(format!("GPIO new error: {:?}", e)))? };
        PinDriver::output(pin).map_err(|e| BoardError::HalError(format!("PinDriver output error: {:?}", e)))
    }
    fn set_pin_state(&self, pin: &mut Self::OutputPin, state: HalPinState) -> Result<(), BoardError> {
        match state {
            HalPinState::High => pin.set_high().map_err(|e| BoardError::HalError(format!("set_high error: {:?}", e))),
            HalPinState::Low => pin.set_low().map_err(|e| BoardError::HalError(format!("set_low error: {:?}", e))),
        }
    }
    fn setup_pin_input(&self, gpio_pin_num: u8) -> Result<Self::InputPin, BoardError> {
        let pin = unsafe { GpioPin::new(gpio_pin_num).map_err(|e| BoardError::HalError(format!("GPIO new error: {:?}", e)))? };
        PinDriver::input(pin).map_err(|e| BoardError::HalError(format!("PinDriver input error: {:?}", e)))
    }
    fn read_pin_state(&self, pin: &Self::InputPin) -> Result<HalPinState, BoardError> {
        if pin.is_high() {
            Ok(HalPinState::High)
        } else {
            Ok(HalPinState::Low)
        }
    }
    fn setup_pwm_pin(&self, gpio_pin_num: u8) -> Result<Self::PwmPin, BoardError> {
        let timer = LedcTimerDriver::new(
            LedcTimer::Timer0,
            &LedcTimerConfig::new().frequency(1000.Hz()).resolution(LedcResolution::Bits8),
        ).map_err(|e| BoardError::HalError(format!("LEDC timer error: {:?}", e)))?;
        let channel = LedcChannel::Channel0;
        LedcChannelDriver::new(channel, timer, unsafe { GpioPin::new(gpio_pin_num).unwrap() })
            .map_err(|e| BoardError::HalError(format!("LEDC channel error: {:?}", e)))
    }
    fn set_pwm_duty_cycle(&self, pin: &mut Self::PwmPin, duty_cycle_percent: f32) -> Result<(), BoardError> {
        let max_duty = pin.get_max_duty();
        let duty = ((duty_cycle_percent / 100.0) * (max_duty as f32)).round() as u32;
        pin.set_duty(duty).map_err(|e| BoardError::HalError(format!("LEDC set_duty error: {:?}", e)))
    }
    fn cleanup(&self) {
        // Optionally cleanup resources
    }
}
