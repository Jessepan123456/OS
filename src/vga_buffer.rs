//! VGA text buffer implementation.
//! 
//! Provides kernel-level printing through the VGA text
//! Custom print!/println! macros.

// Help let the complier know that they shouldn't remove anything uneeded
use volatile::Volatile;
use core::fmt;
// Lazy static, compile when first access instead of when it starts
use lazy_static::lazy_static;
// Spinlock: threads siply try to lock it again until it free again
use spin::Mutex;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
//Store it as u8
#[repr(u8)]
/// VGA text mode color values.
/// 
/// Each color corresponds to a 4-bit value used by the VGA hardware.
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
/// Stores the foreground and background colors in one byte.
/// 
/// The upper 4 bits represent the background color and the lower 4 bits
/// represent the foreground color.
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
/// Represent one character cell in VGA text mode.
/// 
/// Each cell contains an ASCII character and its color information.
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
/// VGA text buffer layout
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

/// Provides an interface for writing characters to the VGA buffer.
/// 
/// It keeps track of the current column and hanldes new lines and scrolling
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    /// Handles newline characters separately and writes normal characters
    /// to the bottom row of the VGA buffer.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                //Over the limit
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                //Move Byte to [row][col]
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                //Set the byte
                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    /// Scrolls the VGA buffer up by one row.
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    /// Clears the row by overwriting the characters
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    /// Convert to bytes and them print one by one
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                //printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                //not part of the range
                _ => self.write_byte(0xfe),
            }
        }
    }
}

/// Support for writing int or floats
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

//Previous issue was that color wasn't to convert in compile time
lazy_static! {
    /// Global VGA text writer protected by a spinlock.
    /// 
    /// Provides a single shared writer for kernel printing.
    /// 
    /// Mutex prevents mutiple execution contexts from writing to the VGAbuffer at
    /// the same time.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

/// Internal implemetation for the print! macros.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

/// Internal implemetation for the println! macros.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

/// Verifies that println writes the expected characters into VGA memory
#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}