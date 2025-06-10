pub mod pfpwm;
pub mod esp32;
pub mod orangepi;

pub mod hal;

// Add HAL module declarations, guarded by features
#[cfg(feature = "rpi")]
pub use hal::rpi_hal;
#[cfg(feature = "orangepi")]
pub use hal::orangepi_hal;
#[cfg(feature = "esp32")]
pub use hal::esp32_hal;

// Only import PwmPin, do not alias InputPin/OutputPin/PinState here
pub use embedded_hal::PwmPin as HalPwmPin;

// Reintroduce a local PinState enum for digital pin state abstraction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HalPinState {
    High,
    Low,
}

// Error type for board operations
#[derive(Debug, Clone)] // Ensure BoardError does NOT derive Copy if it holds Strings or other non-Copy types
pub enum BoardError {
    General(String),
    PinMapError(String),
    #[cfg(feature = "rpi")]
    Rpi(crate::boards::rpi_hal::RpiHalError),
    #[cfg(feature = "esp32")]
    Esp32(crate::boards::esp32_hal::EspHalError),
    #[cfg(feature = "orangepi")]
    OrangePi(crate::boards::orangepi_hal::SysfsHalError),
    Dummy(DummyHalError),
    FeatureNotEnabled(String),
    DatabaseError(String),
    UnknownBoard(String),
    IoError(String),
    HalError(String), // <-- add this variant
    PinNotFound(String),
    InvalidPinType(String),
}

// Defines the type/purpose of a pin
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PinType {
    GpioOutput,
    GpioInput,
    Pwm,
    SpiMosi,
    SpiMiso,
    SpiClk,
    SpiCs,
    I2cSda,
    I2cScl,
    UartTx,
    UartRx,
    Unassigned,
    Other(&'static str),
}

// Represents a single pin on the board
#[derive(Debug, Clone)] // Removed Copy as function_name is &'static str which is not always Copy
pub struct Pin {
    pub gpio_pin: u8,                 // The BCM or SoC-specific GPIO number
    pub function_name: &'static str,  // Logical name, e.g., "FAN_RELAY", "SPI_MOSI"
    pub pin_type: PinType,            // Type of the pin
    pub physical_pin: Option<u8>,     // Optional physical header pin number for reference
}

// Trait for board-specific pin mappings
pub trait BoardPinMap {
    fn pin_for_function(&self, function_name: &str) -> Option<Pin>;
    fn get_all_pins(&self) -> Vec<Pin>; // Added this
    fn get_board_name(&self) -> &'static str; // Added this
}

// Trait for platform-specific hardware abstraction (using embedded-hal traits)
// This defines the contract for how to interact with hardware on a specific platform.
pub trait PlatformHal {
    // Associated types for the actual pin objects from the HAL crate
    type OutputPin; // v2 OutputPin has Error
    type InputPin;   // v2 InputPin has Error
    type PwmPin: HalPwmPin<Duty = u16>; // PwmPin (0.2.7) itself doesn't have an Error assoc. type, methods don't return Result.
    type PinError: core::fmt::Debug + Clone;
    // Initializes the HAL, e.g., taking ownership of peripherals
    // fn new() -> Result<Self, BoardError> where Self: Sized; // Might need board-specific params

    // GPIO Output
    fn setup_pin_output(&self, gpio_pin_num: u8) -> Result<Self::OutputPin, BoardError>;
    fn set_pin_state(&self, pin: &mut Self::OutputPin, state: HalPinState) -> Result<(), BoardError>;

    // GPIO Input
    fn setup_pin_input(&self, gpio_pin_num: u8) -> Result<Self::InputPin, BoardError>;
    fn read_pin_state(&self, pin: &Self::InputPin) -> Result<HalPinState, BoardError>; // InputPin read often takes &self

    // PWM Output
    // `channel` might be relevant if a GPIO can be routed to different PWM channels/controllers
    // For now, assume gpio_pin_num is sufficient to identify the PWM resource.
    fn setup_pwm_pin(&self, gpio_pin_num: u8 /*, pwm_channel_id: u8 */) -> Result<Self::PwmPin, BoardError>;
    // Takes duty_cycle_percent (0.0 to 100.0)
    fn set_pwm_duty_cycle(&self, pin: &mut Self::PwmPin, duty_cycle_percent: f32) -> Result<(), BoardError>;
    // fn enable_pwm(&self, pin: &mut Self::PwmPin) -> Result<(), BoardError>;
    // fn disable_pwm(&self, pin: &mut Self::PwmPin) -> Result<(), BoardError>;


    // Cleanup any resources used by the HAL (e.g., unexport GPIOs)
    fn cleanup(&self); // This might not be needed if RAII is used by HAL objects
}

// Dummy HAL for testing without real hardware
#[derive(Debug, Clone)]
pub struct DummyHalError(String);

impl core::fmt::Display for DummyHalError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DummyHalError: {}", self.0)
    }
}

#[derive(Debug)]
pub struct DummyOutputPin {
    pin_num: u8,
}
impl DummyOutputPin {
    fn set_low(&mut self) -> Result<(), DummyHalError> {
        println!("DummyOutputPin[{}]: Set LOW", self.pin_num);
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), DummyHalError> {
        println!("DummyOutputPin[{}]: Set HIGH", self.pin_num);
        Ok(())
    }
}

#[derive(Debug)]
pub struct DummyInputPin {
    pin_num: u8,
}
impl DummyInputPin {
    fn is_high(&self) -> Result<bool, DummyHalError> {
        println!("DummyInputPin[{}]: Read (is_high) -> returning false", self.pin_num);
        Ok(false)
    }
    fn is_low(&self) -> Result<bool, DummyHalError> {
        println!("DummyInputPin[{}]: Read (is_low) -> returning true", self.pin_num);
        Ok(true)
    }
}

#[derive(Debug)]
pub struct DummyPwmPin {
    pin_num: u8,
    duty: u16,
    max_duty: u16,
}

// Implementation for embedded-hal 0.2.7 PwmPin
impl HalPwmPin for DummyPwmPin {
    type Duty = u16; // PwmPin has Duty

    // Methods for 0.2.7 PwmPin do not return Result
    fn disable(&mut self) {
        println!("DummyPwmPin[{}]: Disable", self.pin_num);
    }

    fn enable(&mut self) {
        println!("DummyPwmPin[{}]: Enable", self.pin_num);
    }

    fn get_duty(&self) -> Self::Duty {
        self.duty
    }

    fn get_max_duty(&self) -> Self::Duty {
        self.max_duty 
    }

    fn set_duty(&mut self, duty: Self::Duty) {
        println!("DummyPwmPin[{}]: Set Duty to {}", self.pin_num, duty);
        self.duty = duty;
    }
}


pub struct DummyHal;

impl PlatformHal for DummyHal {
    type OutputPin = DummyOutputPin;
    type InputPin = DummyInputPin;
    type PwmPin = DummyPwmPin;
    type PinError = DummyHalError;

    fn setup_pin_output(&self, gpio_pin_num: u8) -> Result<Self::OutputPin, BoardError> {
        println!("[DummyHal] Setup GPIO {} as Output", gpio_pin_num);
        Ok(DummyOutputPin { pin_num: gpio_pin_num })
    }

    fn set_pin_state(&self, pin: &mut Self::OutputPin, state: HalPinState) -> Result<(), BoardError> {
        println!("[DummyHal] Setting GPIO {} to {:?}", pin.pin_num, state);
        match state {
            HalPinState::High => pin.set_high().map_err(|_| BoardError::HalError("Dummy HAL error".to_string())),
            HalPinState::Low => pin.set_low().map_err(|_| BoardError::HalError("Dummy HAL error".to_string())),
        }
    }

    fn setup_pin_input(&self, gpio_pin_num: u8) -> Result<Self::InputPin, BoardError> {
        println!("[DummyHal] Setup GPIO {} as Input", gpio_pin_num);
        Ok(DummyInputPin { pin_num: gpio_pin_num })
    }

    fn read_pin_state(&self, pin: &Self::InputPin) -> Result<HalPinState, BoardError> {
        println!("[DummyHal] Reading GPIO {}", pin.pin_num);
        // Simulate reading low for simplicity
        // Directly use the HAL's methods which return bool, then map to HalPinState
        if pin.is_low().map_err(|_| BoardError::HalError("Dummy HAL error".to_string()))? {
            Ok(HalPinState::Low)
        } else {
            Ok(HalPinState::High) // Assuming if not low, it's high. Or use is_high().
        }
    }

    fn setup_pwm_pin(&self, gpio_pin_num: u8) -> Result<Self::PwmPin, BoardError> {
        println!("[DummyHal] Setup GPIO {} as PWM", gpio_pin_num);
        Ok(DummyPwmPin { pin_num: gpio_pin_num, duty: 0, max_duty: 1000 }) // Arbitrary max_duty
    }

    fn set_pwm_duty_cycle(&self, pin: &mut Self::PwmPin, duty_cycle_percent: f32) -> Result<(), BoardError> {
        let max_duty = pin.get_max_duty() as f32;
        let duty_val = (duty_cycle_percent / 100.0 * max_duty) as u16;
        println!("[DummyHal] Setting PWM for GPIO {} to {}% (raw value: {})", pin.pin_num, duty_cycle_percent, duty_val);
        pin.set_duty(duty_val);
        Ok(())
    }
    
    fn cleanup(&self) {
        println!("[DummyHal] Cleanup called");
    }
}


// Bridge to interact with hardware using logical pin functions
// This struct will hold instances of both the BoardPinMap and PlatformHal.
#[derive(Debug)] // Added derive Debug
pub struct HardwareBridge<M: BoardPinMap, H: PlatformHal> {
    pub pin_map: M, // Made public for handle_fan_command direct access if needed, or add getter
    hal: H,
    // Future: Store initialized pins here
    // output_pins: std::collections::HashMap<u8, H::OutputPin>,
    // input_pins: std::collections::HashMap<u8, H::InputPin>,
    // pwm_pins: std::collections::HashMap<u8, H::PwmPin>,
}

impl<M: BoardPinMap, H: PlatformHal> HardwareBridge<M, H>
// where <H::OutputPin as HalOutputPin>::Error: std::fmt::Debug, // Add similar bounds if PinError is not Debug
//       <H::InputPin as HalInputPin>::Error: std::fmt::Debug,
//       <H::PwmPin as HalPwmPin>::Error: std::fmt::Debug, // Assuming PwmPin has an Error type
{
    pub fn new(pin_map: M, hal: H) -> Self {
        HardwareBridge { pin_map, hal }
    }

    // Methods now take &mut self to acknowledge potential statefulness for pin management,
    // even if current implementation is stateless (re-initializes pins).
    pub fn set_logical_pin_state(&mut self, function_name: &str, state: HalPinState) -> Result<(), BoardError> {
        let pin_info = self.pin_map.pin_for_function(function_name)
            .ok_or_else(|| BoardError::PinNotFound(function_name.to_string()))?;

        if pin_info.pin_type != PinType::GpioOutput {
            return Err(BoardError::InvalidPinType(format!(
                "Function '{}' is mapped to pin GPIO{} with type {:?}, not GpioOutput.",
                function_name, pin_info.gpio_pin, pin_info.pin_type
            )));
        }

        // Simplified: re-setup pin. A real impl would fetch a stored, initialized pin.
        let mut output_pin = self.hal.setup_pin_output(pin_info.gpio_pin)?;
        self.hal.set_pin_state(&mut output_pin, state)
    }

    // Changed to &mut self for consistency and future stateful pin management.
    pub fn get_logical_pin_state(&mut self, function_name: &str) -> Result<HalPinState, BoardError> {
        let pin_info = self.pin_map.pin_for_function(function_name)
            .ok_or_else(|| BoardError::PinNotFound(function_name.to_string()))?;

        if pin_info.pin_type != PinType::GpioInput {
             return Err(BoardError::InvalidPinType(format!(
                "Function '{}' is mapped to pin GPIO{} with type {:?}, not GpioInput.",
                function_name, pin_info.gpio_pin, pin_info.pin_type
            )));
        }
        let input_pin = self.hal.setup_pin_input(pin_info.gpio_pin)?;
        self.hal.read_pin_state(&input_pin)
    }

    pub fn set_logical_pwm_duty(&mut self, function_name: &str, duty_cycle_percent: f32) -> Result<(), BoardError> {
        let pin_info = self.pin_map.pin_for_function(function_name)
            .ok_or_else(|| BoardError::PinNotFound(function_name.to_string()))?;

        if pin_info.pin_type != PinType::Pwm {
            return Err(BoardError::InvalidPinType(format!(
                "Function '{}' is mapped to pin GPIO{} with type {:?}, not Pwm.",
                function_name, pin_info.gpio_pin, pin_info.pin_type
            )));
        }
        if !(0.0..=100.0).contains(&duty_cycle_percent) { // Validate range
            return Err(BoardError::HalError("Duty cycle must be between 0.0 and 100.0".to_string()));
        }
        let mut pwm_pin = self.hal.setup_pwm_pin(pin_info.gpio_pin)?;
        self.hal.set_pwm_duty_cycle(&mut pwm_pin, duty_cycle_percent)
    }

    pub fn get_board_name(&self) -> &'static str {
        self.pin_map.get_board_name()
    }

    // Example of getting a pin by its function for more direct manipulation if needed
    // pub fn get_output_pin_for_function(&mut self, function: &str) -> Result<P::OutputPin, BoardError> {
    //     let pin_number = self.board_map.pin_for_function(function)?;
    //     self.hal.get_output_pin(pin_number)
    //         .map_err(|e| BoardError::PlatformError(format!("{:?}", e)))
    // }
}

// The old `set_function_state` is now part_of `HardwareBridge`.
// The `pin_provider` closure concept is now formalized by the `PlatformHal` trait
// and its implementations.
//
// pub fn set_function_state<F, P>(
//     pin_function: &str,
//     state: bool,
//     board_map: &dyn BoardPinMap,
//     mut pin_provider: F,
// ) -> Result<(), String>
// where
//     P: embedded_hal::digital::v2::OutputPin,
//     P::Error: std::fmt::Debug,
//     F: FnMut(u8) -> Result<P, String>,
// {
//     match board_map.pin_for_function(pin_function) {
//         Some(pin_num) => {
//             let mut pin = pin_provider(pin_num)?;
//             if state {
//                 pin.set_high().map_err(|e| format!("{:?}", e))?;
//             } else {
//                 pin.set_low().map_err(|e| format!("{:?}", e))?;
//             }
//             Ok(())
//         }
//         None => Err(format!("Pin function '{}' not found in map.", pin_function)),
    pub struct PinMap {
        pub pins: &'static [Pin],
        pub board_name: &'static str,
    }

    impl BoardPinMap for PinMap {
        fn pin_for_function(&self, function_name: &str) -> Option<Pin> {
            self.pins.iter().find(|p| p.function_name == function_name).cloned()
        }
        fn get_all_pins(&self) -> Vec<Pin> {
            self.pins.to_vec()
        }
        fn get_board_name(&self) -> &'static str {
            self.board_name
        }
    }