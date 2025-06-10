// PiFire PWM Board with Raspberry Pi Zero 2 W
use crate::boards::{Pin, PinType};
// use crate::boards::hal::rpi_hal; // If needed for direct HAL access

// Raspberry Pi Zero 2W pin map (BCM numbering, logical function mapping)
// This PIN_MAP is derived from the physical_pin_map() function below.
pub const PIN_MAP: &[Pin] = &[
    Pin { gpio_pin: 0, function_name: "", pin_type: PinType::Unassigned, physical_pin: Some(27) },
    Pin { gpio_pin: 1, function_name: "", pin_type: PinType::Unassigned, physical_pin: Some(28) },
    Pin { gpio_pin: 2, function_name: "I2C_SDA", pin_type: PinType::I2cSda, physical_pin: Some(3) },
    Pin { gpio_pin: 3, function_name: "I2C_SCL", pin_type: PinType::I2cScl, physical_pin: Some(5) },
    Pin { gpio_pin: 4, function_name: "RELAY_PWR", pin_type: PinType::GpioOutput, physical_pin: Some(7) },
    Pin { gpio_pin: 5, function_name: "SPI_LED", pin_type: PinType::GpioOutput, physical_pin: Some(29) },
    Pin { gpio_pin: 6, function_name: "", pin_type: PinType::Unassigned, physical_pin: Some(31) },
    Pin { gpio_pin: 7, function_name: "", pin_type: PinType::Unassigned, physical_pin: Some(26) },
    Pin { gpio_pin: 8, function_name: "SPI_CS", pin_type: PinType::SpiCs, physical_pin: Some(24) },
    Pin { gpio_pin: 9, function_name: "SPI_MISO", pin_type: PinType::SpiMiso, physical_pin: Some(21) },
    Pin { gpio_pin: 10, function_name: "SPI_MOSI", pin_type: PinType::SpiMosi, physical_pin: Some(19) },
    Pin { gpio_pin: 11, function_name: "SPI_SCLK", pin_type: PinType::SpiClk, physical_pin: Some(23) },
    Pin { gpio_pin: 12, function_name: "", pin_type: PinType::Unassigned, physical_pin: Some(32) }, // Was PWM_FAN in a previous version, physical_pin_map() has it unassigned
    Pin { gpio_pin: 13, function_name: "FAN_PWM", pin_type: PinType::Pwm, physical_pin: Some(33) }, // Was PWM_AUGER in a previous version
    Pin { gpio_pin: 14, function_name: "RELAY_AUG", pin_type: PinType::GpioOutput, physical_pin: Some(8) },
    Pin { gpio_pin: 15, function_name: "RELAY_FAN", pin_type: PinType::GpioOutput, physical_pin: Some(10) },
    Pin { gpio_pin: 16, function_name: "ROTARY_CLK", pin_type: PinType::GpioInput, physical_pin: Some(36) },
    Pin { gpio_pin: 17, function_name: "SWITCH_ONOFF", pin_type: PinType::GpioInput, physical_pin: Some(11) },
    Pin { gpio_pin: 18, function_name: "RELAY_IGN", pin_type: PinType::GpioOutput, physical_pin: Some(12) },
    Pin { gpio_pin: 19, function_name: "", pin_type: PinType::Unassigned, physical_pin: Some(35) },
    Pin { gpio_pin: 20, function_name: "ROTARY_DT", pin_type: PinType::GpioInput, physical_pin: Some(38) },
    Pin { gpio_pin: 21, function_name: "ROTARY_SW", pin_type: PinType::GpioInput, physical_pin: Some(40) },
    Pin { gpio_pin: 22, function_name: "", pin_type: PinType::Unassigned, physical_pin: Some(15) },
    Pin { gpio_pin: 23, function_name: "", pin_type: PinType::Unassigned, physical_pin: Some(16) },
    Pin { gpio_pin: 24, function_name: "SPI_DC", pin_type: PinType::GpioOutput, physical_pin: Some(18) },
    Pin { gpio_pin: 25, function_name: "SPI_RST", pin_type: PinType::GpioOutput, physical_pin: Some(22) },
    Pin { gpio_pin: 26, function_name: "FAN", pin_type: PinType::GpioOutput, physical_pin: Some(37) }, // Was RELAY_IGNITER in a previous version
    Pin { gpio_pin: 27, function_name: "TACH", pin_type: PinType::GpioInput, physical_pin: Some(13) },
];

/// Returns a vector of (physical_pin, gpio_number, function) for documentation or UI.
pub fn physical_pin_map() -> Vec<(u8, u8, &'static str)> {
    // Map of physical pin to (gpio, function) for Pi Zero 2W (BCM numbering)
    // This is a simplified mapping for your board; update as needed for your header layout.
    vec![
        (3, 2, "I2C_SDA"),
        (5, 3, "I2C_SCL"),
        (7, 4, "RELAY_PWR"),
        (8, 14, "RELAY_AUG"),
        (10, 15, "RELAY_FAN"),
        (11, 17, "SWITCH_ONOFF"),
        (12, 18, "RELAY_IGN"),
        (13, 27, "TACH"),
        (15, 22, ""),
        (16, 23, ""),
        (18, 24, "SPI_DC"),
        (19, 10, "SPI_MOSI"),
        (21, 9, "SPI_MISO"),
        (22, 25, "SPI_RST"),
        (23, 11, "SPI_SCK"),
        (24, 8, "SPI_CS"),
        (26, 7, ""),
        (27, 0, ""),
        (28, 1, ""),
        (29, 5, "SPI_LED"),
        (31, 6, ""),
        (32, 12, ""),
        (33, 13, "FAN_PWM"),
        (35, 19, ""),
        (36, 16, "ROTARY_CLK"),
        (37, 26, "FAN"),
        (38, 20, "ROTARY_DT"),
        (40, 21, "ROTARY_SW"),
    ]
}

// Export the board name constant for use with the generic PinMap in mod.rs
pub const BOARD_NAME: &str = "PiFire PWM Board (Raspberry Pi, RpiPfpwmPinMap)";

