use crate::*;
use rs_collections::*;

pub struct MemoryStreamWriter {
    data    : Vec<u8>,
    cursor  : usize,
}

impl MemoryStreamWriter {
    pub fn new() -> Self {
        Self { data: Vec::new(), cursor: 0 }
    }

    pub fn data(&self) -> &Vec<u8> { &self.data }
    pub fn data_mut(&mut self) -> &mut Vec<u8> { &mut self.data }
}

impl Drop for MemoryStreamWriter {
    fn drop(&mut self) {}
}

impl Stream for MemoryStreamWriter {
    fn tell(&self) -> usize { self.cursor }
    fn size(&self) -> usize { self.data.len() }
}

impl StreamWriter for MemoryStreamWriter {
    fn write(&mut self, buff: &[u8]) -> Result<usize, ()> {
        let available   = self.data.len() - self.cursor;
        let remaining   = if buff.len() < available { 0 } else { buff.len() - available };
        let first       = usize::min(available, buff.len());

        for i in 0..first {
            self.data[self.cursor + i] = buff[i];
        }
        for i in 0..remaining {
            self.data.push(buff[first + i]);
        }
        self.cursor += buff.len();
        Ok(buff.len())
    }
}

impl StreamSeek for MemoryStreamWriter {
    fn seek(&mut self, cursor: usize) -> Result<usize, ()> {
        if self.data.len() < cursor {
            self.cursor = self.data.len();
        } else {
            self.cursor = cursor;
        }
        Ok(self.cursor)
    }
}

pub struct MemoryStreamReader {
    data    : Vec<u8>,
    cursor  : usize,
}

impl MemoryStreamReader {
    pub fn from(src: &[u8]) -> Self {
        let mut v = Vec::new();
        v.append(src);
        Self { data : v, cursor: 0 }
    }
}

impl Stream for MemoryStreamReader {
    fn tell(&self) -> usize { self.cursor }
    fn size(&self) -> usize { self.data.len() }
}

impl Drop for MemoryStreamReader {
    fn drop(&mut self) {}
}

impl StreamReader for MemoryStreamReader {
    fn is_eof(&self) -> bool { self.cursor == self.data.len() }
    fn read(&mut self, buff: &mut [u8]) -> Result<usize, ()> {
        let read_len =
            if buff.len() > self.data.len() - self.cursor {
                self.data.len() - self.cursor
            } else {
                buff.len()
            };
        for c in 0..read_len {
            buff[c] = self.data[self.cursor + c];
        }
        self.cursor += read_len;
        Ok(read_len)
    }
}

impl StreamSeek for MemoryStreamReader {
    fn seek(&mut self, cursor: usize) -> Result<usize, ()> {
        if self.data.len() < cursor {
            self.cursor = self.data.len();
        } else {
            self.cursor = cursor;
        }
        Ok(self.cursor)
    }
}

///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_memory_write_stream() {
        let mut msw = MemoryStreamWriter::new();
        let string = "hello world".as_bytes();
        assert!(msw.write(string) == Result::Ok(11));
        assert!(msw.tell() == 11);
        assert!(msw.size() == 11);
        let data = msw.data().as_slice();
        assert!(data.len() == 11);
        for i in 0..string.len() {
            assert!(data[i] == string[i]);
        }
        assert!(msw.seek(5) == Result::Ok(5));
        assert!(msw.write("1234".as_bytes()) == Result::Ok(4));
        assert!(msw.tell() == 9);
        let string = "hello1234ld".as_bytes();
        let data = msw.data().as_slice();
        assert!(data.len() == 11);
        for i in 0..string.len() {
            assert!(data[i] == string[i]);
        }
        assert!(msw.seek(14) == Result::Ok(11));
    }

    #[test]
    fn test_memory_read_stream() {
        let mut msr = MemoryStreamReader::from("hello world".as_bytes());
        assert!(msr.tell() == 0);
        assert!(msr.size() == 11);
        let string = "hello world".as_bytes();
        let mut buff = [0u8; 11];
        assert!(msr.read(&mut buff) == Result::Ok(11));
        for i in 0..string.len() {
            assert!(buff[i] == string[i]);
        }
        assert!(msr.seek(5) == Result::Ok(5));

        let mut buff = [0u8; 4];
        assert!(msr.read(&mut buff) == Result::Ok(4));
        assert!(msr.tell() == 9);
        let string = " wor".as_bytes();
        for i in 0..string.len() {
            assert!(buff[i] == string[i]);
        }
        assert!(msr.seek(14) == Result::Ok(11));

        assert!("0.1234".parse::<f32>().unwrap() == 0.1234_f32);
    }
}