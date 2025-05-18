use bitflags::bitflags;
use core::fmt;
use x86_64::instructions::port::Port;

/// A port-mapped UART 16550 serial interface.
pub struct SerialPort {
    data: Port<u8>,
    interrupt_enable: Port<u8>,         //中断使能寄存器
    interrupt_identification: Port<u8>, //中断标识寄存器
    fifo_control: Port<u8>,             //FIFO 控制寄存器
    line_control: Port<u8>,             //线路控制寄存器
    modem_control: Port<u8>,            //调制解调器控制寄存器
    line_status: Port<u8>,              //线路状态寄存器
    modem_status: Port<u8>,             //调制解调器状态寄存器
    scratch: Port<u8>,                  //暂存寄存器
}

bitflags! {
    pub struct LineStatus: u8 {
        const b1 = 0x00;
        const b2 = 0x80;
        const b3 = 0x03;
        const b4 = 0xC7;
        const b5 = 0x0B;
        const b6 = 0x1E;
        const b7 = 0xAE;
        const b8 = 0x0F;
    }
}

// static int init_serial() {
//     outb(PORT + 1, b1);    // Disable all interrupts
//     outb(PORT + 3, LineStatus::b2.bits());    // Enable DLAB (set baud rate divisor)
//     outb(PORT + 0, LineStatus::b3.bits());    // Set divisor to 3 (lo byte) 38400 baud
//     outb(PORT + 1, b1);    //                  (hi byte)
//     outb(PORT + 3, LineStatus::b3.bits());    // 8 bits, no parity, one stop bit
//     outb(PORT + 2, LineStatus::b4.bits());    // Enable FIFO, clear them, with 14-byte threshold
//     outb(PORT + 4, LineStatus::b5.bits());    // IRQs enabled, RTS/DSR set
//     outb(PORT + 4, LineStatus::b6.bits());    // Set in loopback mode, test the serial chip
//     outb(PORT + 0, LineStatus::b7.bits());    // Test serial chip (send byte LineStatus::b7.bits() and check if serial returns same byte)

//     // Check if serial is faulty (i.e: not same byte as sent)
//     if(inb(PORT + 0) != LineStatus::b7.bits()) {
//        return 1;
//     }

//     // If serial is not faulty set it in normal operation mode
//     // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
//     outb(PORT + 4, LineStatus::b8.bits());
//     return 0;
//  }
// send{
//     int is_transmit_empty() {
//         return inb(PORT + 5) & 0x20;
//      }

//      void write_serial(char a) {
//         while (is_transmit_empty() == 0);

//         outb(PORT,a);
//      }
// }
// recieve{
//     int serial_received() {
//         return inb(PORT + 5) & 1;
//      }

//      char read_serial() {
//         while (serial_received() == 0);

//         return inb(PORT);
//      }
// }

impl SerialPort {
    pub const fn new(port: u16) -> Self {
        SerialPort {
            data: Port::new(port),
            interrupt_enable: Port::new(port + 1),
            interrupt_identification: Port::new(port + 2),
            fifo_control: Port::new(port + 2),
            line_control: Port::new(port + 3),
            modem_control: Port::new(port + 4),
            line_status: Port::new(port + 5),
            modem_status: Port::new(port + 6),
            scratch: Port::new(port + 7),
        }
    }

    /// Initializes the serial port.
    pub fn init(&mut self) {
        // FIXME: Initialize the serial port
        unsafe {
            
            self.interrupt_enable.write(LineStatus::b1.bits());
            self.line_control.write(LineStatus::b2.bits());
            self.data.write(LineStatus::b3.bits());
            self.interrupt_enable.write(LineStatus::b1.bits());
            self.line_control.write(LineStatus::b3.bits());
            self.fifo_control.write(LineStatus::b4.bits());
            self.modem_control.write(LineStatus::b5.bits());
            self.modem_control.write(LineStatus::b6.bits());
            self.data.write(LineStatus::b7.bits());
            if self.data.read() != LineStatus::b7.bits() {
                return;
            }
            self.modem_control.write(LineStatus::b8.bits());
            self.interrupt_enable.write(0x01);
        }
    }

    /// Sends a byte on the serial port.
    pub fn send(&mut self, data: u8) {
        // FIXME: Send a byte on the serial port
        unsafe {
            while self.line_status.read() & 0x20 == 0 {}
            self.data.write(data);
        }
    }

    /// Receives a byte on the serial port no wait.
    pub fn receive(&mut self) -> Option<u8> {
        // FIXME: Receive a byte on the serial port no wait
        unsafe {
            if self.line_status.read() & 1 == 0 {
                return None;
            }
            Some(self.data.read())
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}
