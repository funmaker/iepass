use core::slice;
use embedded_io::{ErrorType, Read, ReadExactError, Write};


#[derive(Debug)]
enum WriteState {
    Repeat { len: u8, byte: u8 },
    Literal { len: u8, bytes: [u8; 130] },
}

pub struct Encoder<W: Write> {
    writer: Option<W>,
    state: Option<WriteState>,
}

impl<W: Write> Encoder<W> {
    pub fn new(writer: W) -> Encoder<W> {
        Encoder {
            writer: Some(writer),
            state: None,
        }
    }

    fn write_state(&mut self) -> Result<(), W::Error> {
        let writer = self.writer.as_mut().unwrap();
        
        match self.state.take() {
            None => {}
            Some(WriteState::Repeat { byte, len }) => {
                writer.write_all(&[0x80 | len - 1, byte])?
            }
            Some(WriteState::Literal { bytes, len, .. }) => {
                writer.write_all(&[len - 2])?;
                writer.write_all(&bytes[0..len as usize])?;
            }
        }
        Ok(())
    }

    pub fn finalize(mut self) -> Result<W, W::Error> {
        self.flush()?;
        Ok(self.writer.take().unwrap())
    }
}

impl<W: Write> ErrorType for Encoder<W> {
    type Error = W::Error;
}

impl<W: Write> Write for Encoder<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, W::Error> {
        for &new_byte in buf {
            match self.state {
                // New Byte
                None => {
                    self.state = Some(WriteState::Repeat {
                        len: 1,
                        byte: new_byte,
                    })
                }
                // Append to Repeat
                Some(WriteState::Repeat {
                    len: ref mut len @ ..128,
                    byte,
                }) if byte == new_byte => {
                    *len += 1;
                }
                // Transform singleton repeat into Literal
                Some(WriteState::Repeat { len: 1, byte }) if byte != new_byte => {
                    let mut bytes = [0; 130];
                    bytes[0] = byte;
                    bytes[1] = new_byte;
                    self.state = Some(WriteState::Literal { len: 2, bytes });
                }
                // Split Literal and flush
                Some(WriteState::Literal {
                    len: ref mut len @ 2..,
                    ref mut bytes,
                }) if bytes[*len as usize - 1] == new_byte => {
                    if *len > 2 {
                        *len -= 1;
                    } else {
                        self.state = Some(WriteState::Repeat {
                            len: 1,
                            byte: bytes[0],
                        });
                    }
                    self.write_state()?;
                    self.state = Some(WriteState::Repeat {
                        len: 2,
                        byte: new_byte,
                    });
                }
                // Append to Literal
                Some(WriteState::Literal {
                    len: ref mut len @ 0..129,
                    ref mut bytes,
                }) => {
                    bytes[*len as usize] = new_byte;
                    *len += 1;
                }
                // Flush and start new Repeat
                _ => {
                    self.write_state()?;
                    self.state = Some(WriteState::Repeat {
                        len: 1,
                        byte: new_byte,
                    });
                }
            };
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), W::Error> {
        self.write_state()?;
        self.writer.as_mut().unwrap().flush()?;
        Ok(())
    }
}

impl<W: Write> Drop for Encoder<W> {
    fn drop(&mut self) {
        if self.writer.is_some() {
            let _ = self.flush();
        }
    }
}

enum ReadState {
    Repeat {
        len: usize,
        byte: u8,
    },
    Literal {
        len: usize,
        pos: usize,
        bytes: [u8; 130],
    },
}

pub struct Decoder<R> {
    reader: R,
    state: Option<ReadState>,
}

impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> Decoder<R> {
        Decoder {
            reader,
            state: None,
        }
    }

    fn read_state(&mut self) -> Result<(), R::Error> {
        let mut len = 0;
        match self.reader.read_exact(slice::from_mut(&mut len)) {
            Ok(_) => {}
            Err(ReadExactError::UnexpectedEof) => {
                self.state = None;
                return Ok(());
            }
            Err(ReadExactError::Other(err)) => return Err(err),
        }

        if len < 0x80 {
            let len = len as usize + 2;
            let mut bytes = [0; 130];
            self.reader
                .read_exact(&mut bytes[0..len])
                .map_err(|err| match err {
                    ReadExactError::UnexpectedEof => panic!("Unexpected EOF"),
                    ReadExactError::Other(err) => err,
                })?;
            self.state = Some(ReadState::Literal { bytes, len, pos: 0 });
        } else {
            let len = (len & !0x80) as usize + 1;
            let mut byte = 0;
            self.reader
                .read_exact(slice::from_mut(&mut byte))
                .map_err(|err| match err {
                    ReadExactError::UnexpectedEof => panic!("Unexpected EOF"),
                    ReadExactError::Other(err) => err,
                })?;
            self.state = Some(ReadState::Repeat { byte, len });
        }

        Ok(())
    }
}

impl<R: Read> ErrorType for Decoder<R> {
    type Error = R::Error;
}

impl<R: Read> Read for Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if self.state.is_none() {
            self.read_state()?;
        }

        match self.state {
            None => Ok(0),
            Some(ReadState::Literal {
                ref bytes,
                len,
                ref mut pos,
            }) => {
                let to_be_written = buf.len().min(len - *pos);
                buf[0..to_be_written].copy_from_slice(&bytes[*pos..(*pos + to_be_written)]);

                if *pos + to_be_written >= len {
                    self.state = None;
                } else {
                    *pos += to_be_written;
                }

                Ok(to_be_written)
            }
            Some(ReadState::Repeat { byte, ref mut len }) => {
                let to_be_written = buf.len().min(*len);
                buf[0..to_be_written].fill(byte);

                if to_be_written >= *len {
                    self.state = None;
                } else {
                    *len -= to_be_written;
                }

                Ok(to_be_written)
            }
        }
    }
}

#[cfg(feature = "std")] #[allow(unused_imports)] use std_impls::*;
#[cfg(feature = "std")]
mod std_impls {
    use super::*;
    use std::io::{self, Read, Write};
    
    impl<W: embedded_io::Write> Write for Encoder<W>
        where Self: embedded_io::Write + ErrorType<Error = io::Error> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            embedded_io::Write::write(self, buf)
        }
        
        fn flush(&mut self) -> io::Result<()> {
            embedded_io::Write::flush(self)
        }
    }
    
    impl<W: embedded_io::Read> Read for Decoder<W>
    where Self: embedded_io::Read + ErrorType<Error = io::Error> {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            embedded_io::Read::read(self, buf)
        }
    }
    
    pub struct WriteWrap<W>(W);
    pub struct ReadWrap<W>(W);
    
    impl<W: Write> ErrorType for WriteWrap<W> { type Error = io::Error; }
    impl<R: Read> ErrorType for ReadWrap<R> { type Error = io::Error; }
    
    impl<W: Write> embedded_io::Write for WriteWrap<W> {
        fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> { Write::write(&mut self.0, buf) }
        fn flush(&mut self) -> Result<(), Self::Error> { Write::flush(&mut self.0) }
    }
    impl<R: Read> embedded_io::Read for ReadWrap<R>  {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> { Read::read(&mut self.0, buf) }
    }
    
    impl<W: Write> Encoder<WriteWrap<W>> {
        pub fn new_std(writer: W) -> Self {
            Self::new(WriteWrap(writer))
        }
    }
    
    impl<R: Read> Decoder<ReadWrap<R>> {
        pub fn new_std(reader: R) -> Self {
            Self::new(ReadWrap(reader))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;
    
    #[test]
    fn test_encode_decode() {
        let cases = [
            &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16][..],
            &[10; 300][..],
            &[1, 1, 1, 1, 1, 1, 10, 2, 2, 2, 2, 10, 11, 12, 3, 3, 3, 4, 4, 3, 3, 3][..],
            &include_bytes!("../../assets/XD.raw")[..],
            &include_bytes!("../../assets/BadApple.raw")[..],
            &include_bytes!("../../assets/calib1.raw")[..],
            &include_bytes!("../../assets/calib2.raw")[..],
            &include_bytes!("../../assets/calib3.raw")[..],
            &include_bytes!("../../assets/calib4.raw")[..],
            &include_bytes!("../../assets/calib5.raw")[..],
            &include_bytes!("../../assets/calib-target.raw")[..],
        ];

        for case in cases {
            println!("case: {case:?}");

            let mut enc = Encoder::new(Vec::new());
            enc.write_all(case).unwrap();
            let encoded = enc.finalize().unwrap();

            println!("encoded: {encoded:?}");

            let mut decoded = Vec::new();
            let mut buf = [0; 128];
            let mut dec = Decoder::new(&*encoded);

            loop {
                let read = dec.read(&mut buf).unwrap();
                if read == 0 {
                    break;
                }
                decoded.extend_from_slice(&buf[..read]);
            }

            println!("decoded: {decoded:?}");

            assert_eq!(&decoded[..], case);
        }
    }
    
    #[test]
    fn test_encoded() {
        static RAW: &[u8] = include_bytes!("../../assets/calib1.raw");
        static SMOL: &[u8] = include_bytes!("../../assets/calib1.smol");
        
        let mut enc = Encoder::new(Vec::new());
        enc.write_all(RAW).unwrap();
        let encoded = enc.finalize().unwrap();
        
        assert_eq!(encoded, SMOL);
        
        fn pixels(smol: &[u8]) -> impl Iterator<Item = u16> {
            let mut decoder = Decoder::new(smol);
            
            std::iter::from_fn(move || {
                let mut value = [0, 0];
                decoder.read_exact(&mut value)
                       .ok()
                       .map(|_| u16::from_le_bytes(value))
            })
        }
        
        let mut framebuffer = vec![0, 128*160];
        
        framebuffer.clear();
        framebuffer.extend(pixels(SMOL));
        
        assert_eq!(framebuffer.len(), 128*160);
    }
}
