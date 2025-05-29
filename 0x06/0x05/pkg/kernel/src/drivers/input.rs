use crossbeam_queue::ArrayQueue;
use lazy_static::lazy_static;
use alloc::string::String;


type Key = u8;

lazy_static! {
    static ref INPUT_BUF: ArrayQueue<Key> = ArrayQueue::new(128);
}

#[inline]
pub fn push_key(key: Key) {
    if INPUT_BUF.push(key).is_err() {
        warn!("Input buffer is full. Dropping key '{:?}'", key);
    }
}

#[inline]
pub fn try_pop_key() -> Option<Key> {
    INPUT_BUF.pop()
}

pub fn pop_key() -> Key {
    loop {
        if let Some(key) = try_pop_key() {
            return key;
        }
    }
}

pub fn get_line () -> String {
    let mut line =String::new();
    loop{
        let one_char=pop_key();
        match one_char{
            13 => {
                println!();
                return line;
            }
            0x08 | 0x7F if !line.is_empty() => {
                print!("\x08\x20\x08");
                line.pop();
            }
            _ =>{
                if is_utf8(one_char) {
                    line.push(check_char(one_char));
                    print!("{}",check_char(one_char));
                }else{
                    line.push(one_char as char);
                    print!("{}",one_char as char);
                }
            }
        }
    }
}

fn is_utf8(ch: u8) -> bool {
    ch & 0x80 == 0 || ch & 0xE0 == 0xC0 || ch & 0xF0 == 0xE0 || ch & 0xF8 == 0xF0
}

/// 将UTF-8编码字节序列转换为Unicode码点
///
/// # 参数
/// * `ch` - UTF-8序列的第一个字节
///
/// # 返回值
/// 解码后的Unicode码点(U+0000 - U+10FFFF)
pub fn to_utf8(ch: u8) -> u32 {
    // 1字节ASCII字符 (0xxxxxxx)
    if ch & 0x80 == 0 {
        return ch as u32;
    }
    
    // 初始化码点和要读取的后续字节数
    let (mut char_utf8, trailing_bytes, mask) = match ch {
        // 2字节序列 (110xxxxx)
        b if b & 0xE0 == 0xC0 => (
            (b & 0x1F) as u32, 
            1, 
            0x1F
        ),
        
        // 3字节序列 (1110xxxx)
        b if b & 0xF0 == 0xE0 => (
            (b & 0x0F) as u32, 
            2, 
            0x0F
        ),
        
        // 4字节序列 (11110xxx)
        b if b & 0xF8 == 0xF0 => (
            (b & 0x07) as u32, 
            3, 
            0x07
        ),
        
        // 无效UTF-8起始字节，返回替换字符
        _ => return 0xFFFD, // Unicode替换字符
    };
    
    // 将第一个字节的有效位移到适当位置
    char_utf8 <<= 6 * trailing_bytes;
    
    // 处理后续字节
    for i in 0..trailing_bytes {
        let trailing = pop_key();
        
        // 验证后续字节格式是否为10xxxxxx
        if trailing & 0xC0 != 0x80 {
            return 0xFFFD; // 无效的UTF-8序列
        }
        
        // 将后续字节的有效位添加到码点
        let shift = 6 * (trailing_bytes - 1 - i);
        char_utf8 |= ((trailing & 0x3F) as u32) << shift;
    }
    
    // 检查码点是否过大或是代理区域
    if char_utf8 > 0x10FFFF || (0xD800..=0xDFFF).contains(&char_utf8) {
        return 0xFFFD; // 超过Unicode范围或代理区
    }
    
    // 检查是否使用了过长编码
    let min_code = match trailing_bytes {
        1 => 0x80,      // 2字节序列最小应该编码U+0080
        2 => 0x800,     // 3字节序列最小应该编码U+0800
        3 => 0x10000,   // 4字节序列最小应该编码U+10000
        _ => 0,
    };
    
    if char_utf8 < min_code {
        return 0xFFFD; // 过长编码
    }
    
    char_utf8
}

fn check_char(ch:u8)->char{
    char::from_u32(to_utf8(ch)).unwrap()
}