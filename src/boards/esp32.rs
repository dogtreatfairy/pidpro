// ESP32 Dev Board Example Pin Map
use crate::boards::{Pin, PinType};
// use crate::boards::hal::esp32_hal; // If needed for direct HAL access

pub const PIN_MAP: &[Pin] = &[
    // ESP32 DevKitC V4 Pinout (Common)
    // Left Side
    Pin { gpio_pin: 0, function_name: "BOOT_BUTTON", pin_type: PinType::GpioInput, physical_pin: Some(2) }, // Usually connected to boot button, also ADC1_CH1, TOUCH1
    Pin { gpio_pin: 1, function_name: "UART0_TXD", pin_type: PinType::UartTx, physical_pin: Some(1) }, // TXD0
    Pin { gpio_pin: 2, function_name: "LED_BUILTIN", pin_type: PinType::GpioOutput, physical_pin: Some(3) }, // Usually connected to an on-board LED, also ADC2_CH2, TOUCH2, HSPI_CS0
    Pin { gpio_pin: 3, function_name: "UART0_RXD", pin_type: PinType::UartRx, physical_pin: Some(3) }, // RXD0
    Pin { gpio_pin: 4, function_name: "GPIO4_DAC2_ADC10", pin_type: PinType::GpioOutput, physical_pin: Some(4) }, // DAC2, ADC1_CH10
    Pin { gpio_pin: 5, function_name: "GPIO5_VSPI_CS0", pin_type: PinType::SpiCs, physical_pin: Some(5) }, // VSPI CS0
    Pin { gpio_pin: 33, function_name: "3V3_1", pin_type: PinType::Other("POWER"), physical_pin: Some(6) }, // 3.3V Power
    Pin { gpio_pin: 32, function_name: "3V3_2", pin_type: PinType::Other("POWER"), physical_pin: Some(7) }, // 3.3V Power (EN on some boards, but typically 3.3V)
    Pin { gpio_pin: 18, function_name: "GPIO18_VSPI_CLK", pin_type: PinType::SpiClk, physical_pin: Some(8) }, // VSPI CLK
    Pin { gpio_pin: 19, function_name: "GPIO19_VSPI_MISO", pin_type: PinType::SpiMiso, physical_pin: Some(9) }, // VSPI MISO
    Pin { gpio_pin: 21, function_name: "GPIO21_I2C_SDA", pin_type: PinType::I2cSda, physical_pin: Some(10) }, // I2C SDA
    Pin { gpio_pin: 22, function_name: "GPIO22_I2C_SCL", pin_type: PinType::I2cScl, physical_pin: Some(11) }, // I2C SCL
    Pin { gpio_pin: 23, function_name: "GPIO23_VSPI_MOSI", pin_type: PinType::SpiMosi, physical_pin: Some(12) }, // VSPI MOSI
    Pin { gpio_pin: 25, function_name: "GPIO25_DAC1_ADC18", pin_type: PinType::GpioOutput, physical_pin: Some(13) }, // DAC1, ADC1_CH18
    Pin { gpio_pin: 26, function_name: "GPIO26_ADC19", pin_type: PinType::GpioInput, physical_pin: Some(14) }, // ADC1_CH19
    Pin { gpio_pin: 27, function_name: "GPIO27_ADC17", pin_type: PinType::GpioInput, physical_pin: Some(15) }, // ADC1_CH17
    Pin { gpio_pin: 12, function_name: "GPIO12_ADC15_HSPI_MISO", pin_type: PinType::GpioInput, physical_pin: Some(16) }, // ADC1_CH15, HSPI MISO (also JTAG TDI)
    Pin { gpio_pin: 13, function_name: "GPIO13_ADC14_HSPI_MOSI", pin_type: PinType::GpioOutput, physical_pin: Some(17) }, // ADC1_CH14, HSPI MOSI (also JTAG TCK)
    Pin { gpio_pin: 14, function_name: "GPIO14_ADC16_HSPI_CLK", pin_type: PinType::GpioOutput, physical_pin: Some(18) }, // ADC1_CH16, HSPI CLK (also JTAG TMS)
    Pin { gpio_pin: 15, function_name: "GPIO15_ADC13_HSPI_CS0", pin_type: PinType::SpiCs, physical_pin: Some(19) }, // ADC1_CH13, HSPI CS0 (also JTAG TDO)
    Pin { gpio_pin: 16, function_name: "GPIO16_UART2_RXD", pin_type: PinType::UartRx, physical_pin: Some(20) }, // UART2 RXD
    Pin { gpio_pin: 17, function_name: "GPIO17_UART2_TXD", pin_type: PinType::UartTx, physical_pin: Some(21) }, // UART2 TXD
    Pin { gpio_pin: 34, function_name: "GPIO34_ADC6_INPUT_ONLY", pin_type: PinType::GpioInput, physical_pin: Some(22) }, // ADC1_CH6 (Input only)
    Pin { gpio_pin: 35, function_name: "GPIO35_ADC7_INPUT_ONLY", pin_type: PinType::GpioInput, physical_pin: Some(23) }, // ADC1_CH7 (Input only)
    Pin { gpio_pin: 36, function_name: "GPIO36_ADC0_SVP_INPUT_ONLY", pin_type: PinType::GpioInput, physical_pin: Some(24) }, // ADC1_CH0 (SVP) (Input only)
    Pin { gpio_pin: 39, function_name: "GPIO39_ADC3_SVN_INPUT_ONLY", pin_type: PinType::GpioInput, physical_pin: Some(25) }, // ADC1_CH3 (SVN) (Input only)
    Pin { gpio_pin: 100, function_name: "GND_1", pin_type: PinType::Other("POWER"), physical_pin: Some(26) }, // GND
    Pin { gpio_pin: 101, function_name: "GND_2", pin_type: PinType::Other("POWER"), physical_pin: Some(27) }, // GND
    Pin { gpio_pin: 6, function_name: "FLASH_CLK", pin_type: PinType::Other("FLASH"), physical_pin: Some(28) }, // SPI Flash CLK
    Pin { gpio_pin: 7, function_name: "FLASH_Q_D1", pin_type: PinType::Other("FLASH"), physical_pin: Some(29) }, // SPI Flash Q (D1)
    Pin { gpio_pin: 8, function_name: "FLASH_D_D0", pin_type: PinType::Other("FLASH"), physical_pin: Some(30) }, // SPI Flash D (D0)
    Pin { gpio_pin: 9, function_name: "FLASH_HD_D3", pin_type: PinType::Other("FLASH"), physical_pin: Some(31) }, // SPI Flash HD (D3)
    Pin { gpio_pin: 10, function_name: "FLASH_WP_D2", pin_type: PinType::Other("FLASH"), physical_pin: Some(32) }, // SPI Flash WP (D2)
    Pin { gpio_pin: 11, function_name: "FLASH_CS0_CMD", pin_type: PinType::Other("FLASH"), physical_pin: Some(33) }, // SPI Flash CS0 / CMD
    Pin { gpio_pin: 102, function_name: "GND_3", pin_type: PinType::Other("POWER"), physical_pin: Some(34) }, // GND
    Pin { gpio_pin: 103, function_name: "VIN_5V", pin_type: PinType::Other("POWER"), physical_pin: Some(35) }, // 5V Power In
    // Define RELAY_FAN and RELAY_AUG, RELAY_IGN, PWM_FAN based on typical ESP32 usage
    // These are examples and should be verified for your specific ESP32 board/module
    Pin { gpio_pin: 2, function_name: "RELAY_FAN", pin_type: PinType::GpioOutput, physical_pin: Some(2) }, // Using GPIO2 as an example for Fan Relay
    Pin { gpio_pin: 4, function_name: "RELAY_AUG", pin_type: PinType::GpioOutput, physical_pin: Some(4) }, // Using GPIO4 as an example for Auger Relay
    Pin { gpio_pin: 5, function_name: "RELAY_IGN", pin_type: PinType::GpioOutput, physical_pin: Some(5) }, // Using GPIO5 as an example for Igniter Relay
    Pin { gpio_pin: 18, function_name: "PWM_FAN", pin_type: PinType::Pwm, physical_pin: Some(8) }, // Using GPIO18 as an example for PWM Fan
];

pub const BOARD_NAME: &str = "ESP32 Dev Board (Esp32EspressifPinMap)";
