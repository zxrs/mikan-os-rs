#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct DescriptorType(u16);

#[allow(non_snake_case)]
#[allow(unused)]
impl DescriptorType {
    const UPPER_BYTES: u16 = 0;
    const LDT: u16 = 2;
    const TSS_AVAILABLE: u16 = 9;
    const TSS_BUSY: u16 = 11;
    const CALL_GATE: u16 = 12;
    const INTERRUPT_GATE: u16 = 14;
    const TRAP_GATE: u16 = 15;

    const READ_WRITE: u16 = 2;
    const EXECUTE_READ: u16 = 10;

    pub fn UpperBytes() -> Self {
        Self(Self::UPPER_BYTES)
    }

    pub fn Ldt() -> Self {
        Self(Self::LDT)
    }

    pub fn TssAvailable() -> Self {
        Self(Self::TSS_AVAILABLE)
    }

    pub fn TssBusy() -> Self {
        Self(Self::TSS_BUSY)
    }

    pub fn CallGate() -> Self {
        Self(Self::CALL_GATE)
    }

    pub fn InterruptGate() -> Self {
        Self(Self::INTERRUPT_GATE)
    }

    pub fn TrapGate() -> Self {
        Self(Self::TRAP_GATE)
    }

    pub fn ReadWrite() -> Self {
        Self(Self::READ_WRITE)
    }

    pub fn ExecuteRead() -> Self {
        Self(Self::EXECUTE_READ)
    }

    pub fn to_u16(self) -> u16 {
        self.0
    }
}

impl From<u16> for DescriptorType {
    fn from(value: u16) -> Self {
        DescriptorType(value)
    }
}
