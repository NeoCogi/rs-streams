#![no_std]
pub mod memorystream;
pub mod file;


pub use file::*;
pub use memorystream::*;

pub trait Stream : Drop {
    /// get the current position
    fn tell(&self) -> usize;

    /// get the size of the stream
    fn size(&self) -> usize;
}

pub trait StreamReader : Stream {
    fn read(&mut self, buff: &mut [u8]) -> Result<usize, ()>;
    fn is_eof(&self) -> bool;
}

pub trait StreamWriter : Stream {
    fn write(&mut self, buff: &[u8]) -> Result<usize, ()>;
}

pub trait StreamSeek : Stream {
    fn seek(&mut self, cursor: usize) -> Result<usize, ()>;
}
