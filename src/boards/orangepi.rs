// Orange Pi Zero 2 Pin Map (example, update as needed for your board)
// NOTE: This PinMap is a logical mapping. Control code should always reference logical names (e.g., "RELAY_FAN").
// The correct physical pin for the current board will be used automatically.
// Example: PinMap.RELAY_FAN = on will enable the correct pin for the selected board.

use crate::boards::{Pin, PinType};
// use crate::boards::hal::orangepi_hal; // If needed for direct HAL access

// Example Orange Pi Zero 2W pin map (logical function mapping)
pub const PIN_MAP: &[Pin] = &[
    Pin { gpio_pin: 0, function_name: "I2C_SDA", pin_type: PinType::I2cSda, physical_pin: Some(3) },
    Pin { gpio_pin: 1, function_name: "I2C_SCL", pin_type: PinType::I2cScl, physical_pin: Some(5) },
    Pin { gpio_pin: 2, function_name: "UART_TX", pin_type: PinType::UartTx, physical_pin: Some(8) },
    Pin { gpio_pin: 3, function_name: "UART_RX", pin_type: PinType::UartRx, physical_pin: Some(10) },
    Pin { gpio_pin: 4, function_name: "RELAY_PWR", pin_type: PinType::GpioOutput, physical_pin: Some(7) },
    Pin { gpio_pin: 5, function_name: "SPI_LED", pin_type: PinType::GpioOutput, physical_pin: Some(29) },
    Pin { gpio_pin: 6, function_name: "RELAY_FAN", pin_type: PinType::GpioOutput, physical_pin: Some(12) },
    Pin { gpio_pin: 7, function_name: "RELAY_AUG", pin_type: PinType::GpioOutput, physical_pin: Some(16) },
    Pin { gpio_pin: 8, function_name: "RELAY_IGN", pin_type: PinType::GpioOutput, physical_pin: Some(18) },
    Pin { gpio_pin: 9, function_name: "FAN_PWM", pin_type: PinType::Pwm, physical_pin: Some(33) },
    Pin { gpio_pin: 10, function_name: "FAN", pin_type: PinType::GpioOutput, physical_pin: Some(37) },
    Pin { gpio_pin: 11, function_name: "TACH", pin_type: PinType::GpioInput, physical_pin: Some(13) },
    Pin { gpio_pin: 12, function_name: "ROTARY_CLK", pin_type: PinType::GpioInput, physical_pin: Some(36) },
    Pin { gpio_pin: 13, function_name: "ROTARY_DT", pin_type: PinType::GpioInput, physical_pin: Some(38) },
    Pin { gpio_pin: 14, function_name: "ROTARY_SW", pin_type: PinType::GpioInput, physical_pin: Some(40) },
    Pin { gpio_pin: 15, function_name: "SWITCH_ONOFF", pin_type: PinType::GpioInput, physical_pin: Some(11) },
];

pub const BOARD_NAME: &str = "Orange Pi Zero 2 (OpiZeroPinMap)";
