use crate::error::Result;
use std::{
    io::{BufWriter, Write},
    net::TcpStream,
};

// Writes a message as bytes to the connected node
pub fn w(writer: &mut BufWriter<TcpStream>, msg: &str) -> Result<()> {
    writer.write_all(msg.as_bytes())?;
    writer.flush()?;

    Ok(())
}
