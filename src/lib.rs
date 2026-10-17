use std::io;

use io::BufRead;

use io::BufWriter;
use io::Write;

pub fn brdr2delimited2offsets<R>(
    mut brdr: R,
    split: u8,
) -> impl Iterator<Item = Result<u32, io::Error>>
where
    R: BufRead,
{
    let mut offset: u32 = 0;

    std::iter::from_fn(move || {
        let rslt: Result<Option<u32>, io::Error> = (|| {
            let sz: usize = brdr.skip_until(split)?;
            let u: u32 = sz
                .try_into()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "too many strings"))?;
            match u {
                0 => Ok(None),
                _ => Ok(Some(u)),
            }
        })();
        match rslt {
            Err(e) => Some(Err(e)),
            Ok(None) => None,
            Ok(Some(u)) => {
                offset = offset.saturating_add(u);
                Some(Ok(offset))
            }
        }
    })
}

pub fn u2bytes_le(u: u32) -> [u8; 4] {
    u.to_le_bytes()
}

pub fn u2bytes_be(u: u32) -> [u8; 4] {
    u.to_be_bytes()
}

pub fn offsets2sink<I, S>(offsets: I, mut sink: S) -> Result<(), io::Error>
where
    I: Iterator<Item = Result<u32, io::Error>>,
    S: FnMut(u32) -> Result<(), io::Error>,
{
    for ro in offsets {
        let o: u32 = ro?;
        sink(o)?;
    }
    Ok(())
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Endian {
    #[default]
    Lit,
    Big,
}

impl Endian {
    pub fn to_converter(self) -> impl Fn(u32) -> [u8; 4] {
        match self {
            Self::Lit => u2bytes_le,
            Self::Big => u2bytes_be,
        }
    }
}

pub const SPLIT_DEFAULT: u8 = b'\n';

pub struct Config {
    pub split: u8,
    pub endian: Endian,
}

impl Default for Config {
    fn default() -> Self {
        let split: u8 = SPLIT_DEFAULT;
        Self {
            split,
            endian: Endian::default(),
        }
    }
}

impl Config {
    pub fn brdr2offsets<R>(&self, brdr: R) -> impl Iterator<Item = Result<u32, io::Error>>
    where
        R: BufRead,
    {
        brdr2delimited2offsets(brdr, self.split)
    }
}

impl Config {
    pub fn offsets2raw_sink<I, S>(&self, offsets: I, mut sink: S) -> Result<(), io::Error>
    where
        I: Iterator<Item = Result<u32, io::Error>>,
        S: FnMut(&[u8]) -> Result<(), io::Error>,
    {
        let conv = self.endian.to_converter();
        offsets2sink(offsets, move |u: u32| {
            let converted: [u8; 4] = conv(u);
            sink(&converted)
        })
    }
}

impl Config {
    pub fn brdr2wtr<R, W>(&self, brdr: R, mut wtr: W) -> Result<(), io::Error>
    where
        R: BufRead,
        W: Write,
    {
        let offsets = self.brdr2offsets(brdr); // iter<u32?>
        self.offsets2raw_sink(offsets, |u: &[u8]| wtr.write_all(u))?;
        wtr.flush()
    }
}

impl Config {
    pub fn stdin2stdout(&self) -> Result<(), io::Error> {
        let o = io::stdout();
        let mut ol = o.lock();
        self.brdr2wtr(io::stdin().lock(), BufWriter::new(&mut ol))?;
        ol.flush()
    }
}

#[cfg(test)]
mod tests {
    mod brdr2delimited2offsets {

        #[test]
        fn empty() {
            let input: &[u8] = &[];
            let splited = crate::brdr2delimited2offsets(input, 0);
            let collected: Vec<_> = splited.collect();
            assert!(collected.is_empty());
        }

        #[test]
        fn single() {
            let input: &[u8] = b"helo\0";
            let splited = crate::brdr2delimited2offsets(input, 0);
            let collected: Vec<_> = splited.collect();
            assert_eq!(1, collected.len());
            let r1st: &Result<u32, _> = &collected[0];
            let u1st: u32 = r1st.as_ref().map(|u| *u).unwrap();
            assert_eq!(5, u1st);
        }

        #[test]
        fn double() {
            let input: &[u8] = b"helo\0world\0";
            let splited = crate::brdr2delimited2offsets(input, 0);
            let collected: Vec<Result<u32, _>> = splited.collect();
            assert_eq!(2, collected.len());
            let v0: Option<u32> = collected[0].as_ref().ok().copied();
            let v1: Option<u32> = collected[1].as_ref().ok().copied();

            assert_eq!(Some(5), v0);
            assert_eq!(Some(5 + 6), v1);
        }

        #[test]
        fn triple() {
            let input: &[u8] = b"helo\0world\0hi\0";
            let splited = crate::brdr2delimited2offsets(input, 0);
            let collected: Vec<Result<u32, _>> = splited.collect();
            assert_eq!(3, collected.len());
            let v0: Option<u32> = collected[0].as_ref().ok().copied();
            let v1: Option<u32> = collected[1].as_ref().ok().copied();
            let v2: Option<u32> = collected[2].as_ref().ok().copied();

            assert_eq!(Some(5), v0);
            assert_eq!(Some(5 + 6), v1);
            assert_eq!(Some(5 + 6 + 3), v2);
        }

        #[test]
        fn empty_string() {
            let input: &[u8] = b"helo\0\0hi\0";
            let splited = crate::brdr2delimited2offsets(input, 0);
            let collected: Vec<Result<u32, _>> = splited.collect();
            assert_eq!(3, collected.len());
            let v0: Option<u32> = collected[0].as_ref().ok().copied();
            let v1: Option<u32> = collected[1].as_ref().ok().copied();
            let v2: Option<u32> = collected[2].as_ref().ok().copied();

            assert_eq!(Some(5), v0);
            assert_eq!(Some(5 + 1), v1);
            assert_eq!(Some(5 + 1 + 3), v2);
        }
    }
}
