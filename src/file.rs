use crate::*;
use rs_mem::*;
use rs_collections::*;
use libc::*;

pub struct FileWriter {
    file    : *mut ::libc::FILE,
}

impl FileWriter {
    pub fn create(fname: &str) -> Result<Self, ()> {
        let mut sname = String::from(fname);
        sname.push('\0' as u8);
        let f = unsafe { ::libc::fopen(sname.as_str().as_bytes().as_ptr() as *const i8, "wb\0".as_bytes().as_ptr() as *const i8) };
        if f as * const _ == ::core::ptr::null() {
            Result::Err(())
        } else {
            Result::Ok(Self { file: f })
        }
    }

    pub fn write(&mut self, bytes: &[u8]) -> Result<usize, ()> {
        let count = unsafe {  ::libc::fwrite(bytes.as_ptr() as *const c_void, 1, bytes.len(), self.file) };
        if count < bytes.len() {
            if unsafe { ::libc::ferror(self.file) } != 0 {
                Result::Err(())
            } else {
                Result::Ok(count)
            }
        } else {
            Result::Ok(count)
        }
    }

    pub fn tell(&self) -> usize { unsafe { ::libc::ftell(self.file) as usize } }
    pub fn size(&self) -> usize {
        unsafe {
            let curr = self.tell();
            ::libc::fseek(self.file, 0, ::libc::SEEK_END);
            let size = self.tell();
            ::libc::fseek(self.file, curr as i64, ::libc::SEEK_SET);
            size
        }
    }
}

impl Drop for FileWriter {
    fn drop(&mut self) {
        unsafe { ::libc::fclose(self.file) };
    }
}

impl Stream for FileWriter {
    fn tell(&self) -> usize { self.tell() }
    fn size(&self) -> usize { self.size() }
}

impl StreamSeek for FileWriter {
    fn seek(&mut self, pos: usize) -> Result<usize, ()> {
        unsafe { ::libc::fseek(self.file, pos as i64, ::libc::SEEK_SET) };
        Result::Ok(self.tell())
    }
}

impl StreamWriter for FileWriter {
    fn write(&mut self, buff: &[u8]) -> Result<usize, ()> {
        self.write(buff)
    }
}

////////////////////////////////////////////////////////////////////////////////
pub struct FileReader {
    file    : *mut ::libc::FILE,
}

impl FileReader {
    pub fn open(fname: &str) -> Result<Self, ()> {
        let mut sname = String::from(fname);
        sname.push(b'\0');
        let f = unsafe { ::libc::fopen(sname.as_str().as_bytes().as_ptr() as *const i8, "rb\0".as_bytes().as_ptr() as *const i8) };
        if f as * const _ == ::core::ptr::null() {
            Result::Err(())
        } else {
            Result::Ok(Self { file: f })
        }
    }

    pub fn tell(&self) -> usize { unsafe { ::libc::ftell(self.file) as usize } }
    pub fn size(&self) -> usize {
        unsafe {
            let curr = self.tell();
            ::libc::fseek(self.file, 0, ::libc::SEEK_END);
            let size = self.tell();
            ::libc::fseek(self.file, curr as i64, ::libc::SEEK_SET);
            size
        }
    }

    pub fn read(&mut self, buff: &mut [u8]) -> Result<usize, ()> {
        let count = unsafe {  ::libc::fread(buff.as_mut_ptr() as *mut c_void, 1, buff.len(), self.file) };
        if count < buff.len() {
            if unsafe { ::libc::ferror(self.file) } != 0 {
                Result::Err(())
            } else {
                Result::Ok(count)
            }
        } else {
            Result::Ok(count)
        }
    }

    pub fn read_line(&mut self, buff: &mut [u8]) -> Result<usize, ()> {
        let n = unsafe { ::libc::fgets(buff.as_mut_ptr() as *mut i8, buff.len() as c_int, self.file) };
        Result::Ok(n as usize)
    }
}


impl Drop for FileReader {
    fn drop(&mut self) {
        unsafe { ::libc::fclose(self.file) };
    }
}

impl Stream for FileReader {
    fn tell(&self) -> usize { self.tell() }
    fn size(&self) -> usize { self.size() }
}

impl StreamSeek for FileReader {
    fn seek(&mut self, pos: usize) -> Result<usize, ()> {
        unsafe { ::libc::fseek(self.file, pos as i64, ::libc::SEEK_SET) };
        Result::Ok(self.tell())
    }
}

impl StreamReader for FileReader {
    fn read(&mut self, buff: &mut [u8]) -> Result<usize, ()> {
        self.read(buff)
    }

    fn is_eof(&self) -> bool {
        if unsafe { ::libc::feof(self.file) } != 0 {
            true
        } else {
            false
        }
    }
}

pub trait StreamSeek : Stream {
    fn seek(&mut self, cursor: usize) -> Result<usize, ()>;
}

////////////////////////////////////////////////////////////////////////////////
pub struct File {}

impl File {
    pub fn exist(fname: &str) -> bool {
        let f = FileReader::open(fname);
        match f {
            Ok(_) => true,
            _ => false,
        }
    }

    pub fn remove(fname: &str) -> Result<(), ()> {
        let mut sname = String::from(fname);
        sname.push(b'\0');
        let f = unsafe { ::libc::remove(sname.as_str().as_bytes().as_ptr() as *const i8) };
        if f != 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    pub fn rename(old_name: &str, new_name: &str) -> Result<(), ()> {
        let mut o = String::from(old_name);
        o.push(b'\0');
        let mut n = String::from(new_name);
        n.push(b'\0');
        let f = unsafe { ::libc::rename(o.as_str().as_bytes().as_ptr() as *const i8, n.as_str().as_bytes().as_ptr() as *const i8) };
        if f != 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    pub fn tmpname() -> String {
        unsafe {
            let s = alloc_array::<i8>(::libc::L_tmpnam as usize);
            ::libc::tmpnam(s);
            let len = ::libc::strlen(s);
            let slice = ::core::slice::from_raw_parts(s as *const u8, len);
            let st = String::from(&::core::str::from_utf8(slice).unwrap());
            free_array_ptr(s, ::libc::L_tmpnam as usize);
            st
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn fprint_strings<'a>(f: *mut libc::FILE, arr: &[&'a str]) {
    for s in arr {
        unsafe { libc::fprintf(f,"%.*s\0".as_bytes().as_ptr() as *const i8, s.len(), s.as_bytes().as_ptr())};
    }
}

#[macro_export]
macro_rules! fprint {
    () => {{}};
    ($stream:expr, $arg:expr) => { fprint!($stream, "{}", $arg) };
    ($stream:expr, $($args:expr),+) => { unsafe { $crate::fprint_strings($stream, &[rs_collections::format!($($args),+).as_str()]) } };
}

#[macro_export]
macro_rules! fprintln {
    () => {};
    ($stream:expr, $arg:expr) => { fprintln!($stream, "{}", $arg) };
    ($stream:expr, $($args:expr),+) => { unsafe { $crate::fprint_strings($stream, &[rs_collections::format!($($args),+).as_str(), "\n"]) } };
}

#[macro_export]
macro_rules! print {
    () => {};
    ($($args:expr),+) => { fprint!($crate::console::stdout, $($args),+) };
}

#[macro_export]
macro_rules! println {
    () => {};
    ($($args:expr),+) => { fprintln!($crate::console::stdout, $($args),+) };
}

#[macro_export]
macro_rules! error {
    () => {};
    ($($args:expr),+) => { fprint!($crate::console::stderr, $($args),+) };
}

#[macro_export]
macro_rules! errorln {
    () => {};
    ($($args:expr),+) => { fprintln!($crate::console::stderr, $($args),+) };
}

////////////////////////////////////////////////////////////////////////////////
pub mod console {
    #[link(name = "c")]
    extern "C"
    {
        pub static mut stdin: *mut libc::FILE;

        pub static mut stdout: *mut libc::FILE;

        pub static mut stderr: *mut libc::FILE;
    }

    pub fn read_line(s: &mut [u8]) -> usize {
        unsafe { ::libc::strlen(::libc::fgets(s.as_mut_ptr() as *mut i8, s.len() as super::c_int, stdin)) as usize }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use core::{str};
    use super::*;
    #[test]
    fn test_create_read_remove_file() {
        let name = File::tmpname();
        {
            let mut f = FileWriter::create(name.as_str());
            let s = "Hello File";
            match &mut f {
                Ok(f) => {
                    let res = f.write(s.as_bytes());
                    assert!(res.unwrap() == s.as_bytes().len());
                    assert!(File::exist(name.as_str()));
                },
                _ => panic!("couldn't create file!")
            }
        }

        {
            let mut f = FileReader::open(name.as_str());
            let s = "Hello File";
            match &mut f {
                Ok(f) => {
                    assert!(f.size() == 10);
                    let mut buff : [u8; 10] = [0; 10];
                    let res = f.read(&mut buff);
                    assert!(res.unwrap() == s.as_bytes().len());
                    assert!(str::from_utf8(&buff).unwrap() == s);
                },
                _ => panic!("couldn't open file!")
            }
        }

        {
            File::remove(name.as_str()).unwrap();
        }
    }

    #[test]
    fn test_macros() {
        print!(123);
        println!(456);
        print!("hello world {} {}", 1, 2);
        println!("hello world {} {}", 3, 4);

        error!(123);
        errorln!(456);
        error!("hello world {} {}", 1, 2);
        errorln!("hello world {} {}", 3, 4);
    }
}