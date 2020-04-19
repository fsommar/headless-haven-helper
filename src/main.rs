use std::convert::TryInto;
use std::io::prelude::*;
use std::net::TcpStream;

mod state;

use state::State;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(std::env::var("HAVEN_HELPER_SERVER")?)?;

    let mut buf = [0; 1024];
    loop {
        stream.read_exact(&mut buf[..2])?;
        let str_len = i16::from_be_bytes(buf[..2].try_into().unwrap()) as usize;
        stream.read_exact(&mut buf[..str_len])?;
        let _state_str = std::str::from_utf8(&buf[..str_len]).unwrap();

        let mut varint_buf = [0u8; 5];
        stream.peek(&mut varint_buf)?;
        let (varint_len, data_len) =
            state::read_varint(&varint_buf).ok_or("unable to read data len header")?;

        stream.read_exact(&mut buf[..varint_len + data_len as usize])?;
        let mut data = vec![0u8; data_len as usize];
        data.copy_from_slice(&buf[varint_len..varint_len + data_len as usize]);
        if !data.is_empty() {
            let message_number = i32::from_be_bytes(data[..4].try_into()?);
            println!("message number {}", message_number);
            let mut file = std::fs::File::create(format!("{}-state.bin", message_number))?;
            file.write_all(&data[4..])?;
            let state: State = state::from_bytes(&data[4..])?;
            println!("{:#?}", state);
        }
    }
}
