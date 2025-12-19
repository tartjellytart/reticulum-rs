use crate::error::Result;

#[derive(Debug, Clone)]
pub struct SerialConfig {
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: StopBits,
    pub parity: Parity,
    pub flow_control: FlowControl,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopBits {
    One = 1,
    Two = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parity {
    None,
    Even,
    Odd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowControl {
    None,
    Hardware,
    Software,
}

pub trait SerialDriver: Send + Sync {
    fn open(&mut self) -> Result<()>;
    fn close(&mut self) -> Result<()>;
    fn is_open(&self) -> bool;
    fn write(&mut self, data: &[u8]) -> Result<usize>;
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;
    fn bytes_available(&self) -> Result<usize>;
    fn set_dtr(&mut self, state: bool) -> Result<()>;
    fn set_rts(&mut self, state: bool) -> Result<()>;
}

