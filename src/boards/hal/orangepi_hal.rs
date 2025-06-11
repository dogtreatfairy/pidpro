// Orange Pi HAL implementation using linux-embedded-hal for GPIO, I2C, and SPI
use crate::boards::{HalPinState, BoardError, PlatformHal};
use linux_embedded_hal::{SysfsPin, Spidev, I2cdev};

pub struct LinuxOrangePiHal {
    pub i2c: I2cdev,
    pub spi: Spidev,
}

impl LinuxOrangePiHal {
    pub fn new() -> Result<Self, BoardError> {
        let i2c = I2cdev::new("/dev/i2c-1").map_err(|e| BoardError::HalError(format!("I2C init error: {:?}", e)))?;
        let spi = Spidev::open("/dev/spidev0.0").map_err(|e| BoardError::HalError(format!("SPI init error: {:?}", e)))?;
        Ok(LinuxOrangePiHal { i2c, spi })
    }
    pub fn i2c(&mut self) -> &mut I2cdev {
        &mut self.i2c
    }
    pub fn spi(&mut self) -> &mut Spidev {
        &mut self.spi
    }
}

impl PlatformHal for LinuxOrangePiHal {
    type OutputPin = SysfsPin;
    type InputPin = SysfsPin;
    type PwmPin = ();
    type PinError = linux_embedded_hal::sysfs_gpio::Error;

    fn setup_pin_output(&self, gpio_pin_num: u8) -> Result<Self::OutputPin, BoardError> {
        let pin = SysfsPin::new(gpio_pin_num as u64);
        pin.export().map_err(|e| BoardError::HalError(format!("GPIO export error: {:?}", e)))?;
        pin.set_direction(linux_embedded_hal::sysfs_gpio::Direction::Out)
            .map_err(|e| BoardError::HalError(format!("GPIO set_direction error: {:?}", e)))?;
        Ok(pin)
    }
    fn set_pin_state(&self, pin: &mut Self::OutputPin, state: HalPinState) -> Result<(), BoardError> {
        let value = match state {
            HalPinState::High => 1,
            HalPinState::Low => 0,
        };
        pin.set_value(value).map_err(|e| BoardError::HalError(format!("GPIO set_value error: {:?}", e)))
    }
    fn setup_pin_input(&self, gpio_pin_num: u8) -> Result<Self::InputPin, BoardError> {
        let pin = SysfsPin::new(gpio_pin_num as u64);
        pin.export().map_err(|e| BoardError::HalError(format!("GPIO export error: {:?}", e)))?;
        pin.set_direction(linux_embedded_hal::sysfs_gpio::Direction::In)
            .map_err(|e| BoardError::HalError(format!("GPIO set_direction error: {:?}", e)))?;
        Ok(pin)
    }
    fn read_pin_state(&self, pin: &Self::InputPin) -> Result<HalPinState, BoardError> {
        let value = pin.get_value().map_err(|e| BoardError::HalError(format!("GPIO get_value error: {:?}", e)))?;
        if value == 1 {
            Ok(HalPinState::High)
        } else {
            Ok(HalPinState::Low)
        }
    }
    fn setup_pwm_pin(&self, _gpio_pin_num: u8) -> Result<Self::PwmPin, BoardError> {
        Err(BoardError::HalError("PWM not supported on Orange Pi via linux-embedded-hal".to_string()))
    }
    fn set_pwm_duty_cycle(&self, _pin: &mut Self::PwmPin, _duty_cycle_percent: f32) -> Result<(), BoardError> {
        Err(BoardError::HalError("PWM not supported on Orange Pi via linux-embedded-hal".to_string()))
    }
    fn cleanup(&self) {
        // Optionally unexport pins or cleanup resources
    }
}
