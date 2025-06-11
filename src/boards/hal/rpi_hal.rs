// Raspberry Pi HAL implementation using rppal
// This is a scaffold. Fill in the details as needed for your board and logic.
use crate::boards::{HalPinState, BoardError, PlatformHal};
#[cfg(feature = "rpi")]
use rppal::gpio::{Gpio, OutputPin, InputPin};
#[cfg(feature = "rpi")]
use rppal::pwm::{Pwm, Channel, Polarity};
#[cfg(feature = "rpi")]
use rppal::i2c::I2c;
#[cfg(feature = "rpi")]
use rppal::spi::{Spi, Bus, SlaveSelect, Mode as SpiMode};

pub struct RppalHal {
    gpio: Gpio,
    i2c: I2c,
    spi: Spi,
}

impl RppalHal {
    pub fn new() -> Result<Self, BoardError> {
        let gpio = Gpio::new().map_err(|e| BoardError::HalError(format!("GPIO init error: {:?}", e)))?;
        let i2c = I2c::new().map_err(|e| BoardError::HalError(format!("I2C init error: {:?}", e)))?;
        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 16_000_000, SpiMode::Mode0)
            .map_err(|e| BoardError::HalError(format!("SPI init error: {:?}", e)))?;
        Ok(RppalHal { gpio, i2c, spi })
    }
    /// Get a mutable reference to the I2C bus (for ADS1115)
    pub fn i2c(&mut self) -> &mut I2c {
        &mut self.i2c
    }
    /// Get a mutable reference to the SPI bus (for ILI9341)
    pub fn spi(&mut self) -> &mut Spi {
        &mut self.spi
    }
}

impl PlatformHal for RppalHal {
    type OutputPin = OutputPin;
    type InputPin = InputPin;
    type PwmPin = Pwm;
    type PinError = rppal::gpio::Error;

    fn setup_pin_output(&self, gpio_pin_num: u8) -> Result<Self::OutputPin, BoardError> {
        self.gpio.get(gpio_pin_num)
            .map_err(|e| BoardError::HalError(format!("GPIO get error: {:?}", e)))?
            .into_output()
            .map_err(|e| BoardError::HalError(format!("GPIO into_output error: {:?}", e)))
    }
    fn set_pin_state(&self, pin: &mut Self::OutputPin, state: HalPinState) -> Result<(), BoardError> {
        match state {
            HalPinState::High => { pin.set_high(); Ok(()) },
            HalPinState::Low => { pin.set_low(); Ok(()) },
        }
    }
    fn setup_pin_input(&self, gpio_pin_num: u8) -> Result<Self::InputPin, BoardError> {
        self.gpio.get(gpio_pin_num)
            .map_err(|e| BoardError::HalError(format!("GPIO get error: {:?}", e)))?
            .into_input()
            .map_err(|e| BoardError::HalError(format!("GPIO into_input error: {:?}", e)))
    }
    fn read_pin_state(&self, pin: &Self::InputPin) -> Result<HalPinState, BoardError> {
        if pin.is_high() {
            Ok(HalPinState::High)
        } else {
            Ok(HalPinState::Low)
        }
    }
    fn setup_pwm_pin(&self, gpio_pin_num: u8) -> Result<Self::PwmPin, BoardError> {
        // Map GPIO to PWM channel as needed. Example uses Channel 0 for all.
        Pwm::with_frequency(Channel::Pwm0, 1000.0, 0.0, Polarity::Normal, true)
            .map_err(|e| BoardError::HalError(format!("PWM error: {:?}", e)))
    }
    fn set_pwm_duty_cycle(&self, pin: &mut Self::PwmPin, duty_cycle_percent: f32) -> Result<(), BoardError> {
        let freq = pin.frequency();
        let duty = (duty_cycle_percent / 100.0).clamp(0.0, 1.0);
        pin.set_duty_cycle(duty).map_err(|e| BoardError::HalError(format!("PWM set_duty_cycle error: {:?}", e)))?;
        pin.set_frequency(freq, duty).map_err(|e| BoardError::HalError(format!("PWM set_frequency error: {:?}", e)))
    }
    fn cleanup(&self) {
        // Optionally unexport pins or cleanup resources
    }
}
// I2C, SPI, and other features can be added as needed by extending the trait and struct.
