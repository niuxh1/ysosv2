#![no_std]

use num_enum::FromPrimitive;

pub mod macros;

#[repr(usize)]
#[derive(Clone, Debug, FromPrimitive)]
pub enum Syscall {
    Read = 0,
    Write = 1,

    GetPid = 39,
    Sem =41,
    Fork = 58,

    Spawn = 59,
    Exit = 60,
    WaitPid = 64,

    ListApp = 65531,
    Stat = 65532,
    Allocate = 65533,
    Deallocate = 65534,
    Time = 65529,
    ListDir=42,
    OpenFile = 43,
    CloseFile = 44,
    Brk = 45,
    #[num_enum(default)]
    Unknown = 65535
    
}
