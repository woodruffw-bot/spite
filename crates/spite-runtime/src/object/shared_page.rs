//! Owned shared pages and prepared atomic memory accesses (25.4, 29.6).
//!
//! A prepared access owns its page through Arc. Resolving/allocating the page
//! happens separately under the backing directory lock. No directory guard or
//! raw pointer reaches the atomic step; pages are never replaced or removed.

use super::ArrayBufferError;
use std::sync::{Arc, atomic::Ordering};

#[cfg(not(target_has_atomic = "64"))]
use std::sync::atomic::AtomicU32 as Word;
#[cfg(target_has_atomic = "64")]
use std::sync::atomic::AtomicU64 as Word;
#[cfg(target_has_atomic = "64")]
type Bits = u64;
#[cfg(not(target_has_atomic = "64"))]
type Bits = u32;

const PAGE_BYTES: usize = 4096;
pub(crate) const WORD_BYTES: usize = std::mem::size_of::<Bits>();

#[derive(Debug)]
pub(super) enum PageBytes {
    Ordinary(Vec<u8>),
    Shared(Arc<SharedPage>),
}

impl PageBytes {
    pub(super) fn new(shared: bool) -> Result<Self, ArrayBufferError> {
        if shared {
            let mut words = Vec::new();
            words
                .try_reserve_exact(PAGE_BYTES / WORD_BYTES)
                .map_err(|_| ArrayBufferError::Capacity)?;
            words.resize_with(PAGE_BYTES / WORD_BYTES, || Word::new(0));
            Ok(Self::Shared(Arc::new(SharedPage { words })))
        } else {
            let mut bytes = Vec::new();
            bytes
                .try_reserve_exact(PAGE_BYTES)
                .map_err(|_| ArrayBufferError::Capacity)?;
            bytes.resize(PAGE_BYTES, 0);
            Ok(Self::Ordinary(bytes))
        }
    }

    pub(super) fn read(&self, offset: usize, output: &mut [u8]) {
        match self {
            Self::Ordinary(bytes) => output.copy_from_slice(&bytes[offset..offset + output.len()]),
            Self::Shared(page) => {
                let mut done = 0;
                while done < output.len() {
                    let position = offset + done;
                    let byte = position % WORD_BYTES;
                    let count = (WORD_BYTES - byte).min(output.len() - done);
                    let value = page.words[position / WORD_BYTES]
                        .load(Ordering::Relaxed)
                        .to_ne_bytes();
                    output[done..done + count].copy_from_slice(&value[byte..byte + count]);
                    done += count;
                }
            }
        }
    }

    pub(super) fn write(&mut self, offset: usize, input: &[u8]) {
        match self {
            Self::Ordinary(bytes) => bytes[offset..offset + input.len()].copy_from_slice(input),
            Self::Shared(page) => {
                let mut done = 0;
                while done < input.len() {
                    let position = offset + done;
                    let byte = position % WORD_BYTES;
                    let count = (WORD_BYTES - byte).min(input.len() - done);
                    let word = &page.words[position / WORD_BYTES];
                    let mut observed = word.load(Ordering::Relaxed);
                    loop {
                        let mut value = observed.to_ne_bytes();
                        value[byte..byte + count].copy_from_slice(&input[done..done + count]);
                        match word.compare_exchange_weak(
                            observed,
                            Bits::from_ne_bytes(value),
                            Ordering::Relaxed,
                            Ordering::Relaxed,
                        ) {
                            Ok(_) => break,
                            Err(current) => observed = current,
                        }
                    }
                    done += count;
                }
            }
        }
    }

    pub(super) fn clear(&mut self, start: usize, end: usize) {
        self.write(start, &[0; PAGE_BYTES][..end - start]);
    }

    pub(super) fn ordinary(&self) -> &[u8] {
        let Self::Ordinary(bytes) = self else {
            unreachable!("private byte snapshot")
        };
        bytes
    }

    pub(super) fn prepare(&self, offset: usize, width: usize) -> PreparedAtomic {
        let Self::Shared(page) = self else {
            unreachable!("shared atomic page")
        };
        let byte = offset % WORD_BYTES;
        let shift = if cfg!(target_endian = "little") {
            byte * 8
        } else {
            (WORD_BYTES - byte - width) * 8
        };
        PreparedAtomic {
            page: page.clone(),
            word: offset / WORD_BYTES,
            shift: shift as u32,
            mask: Bits::MAX >> ((WORD_BYTES - width) * 8),
        }
    }
}

#[derive(Debug)]
pub(super) struct SharedPage {
    words: Vec<Word>,
}

/// An owned, aligned access whose storage preparation has finished.
pub(crate) struct PreparedAtomic {
    page: Arc<SharedPage>,
    word: usize,
    shift: u32,
    mask: Bits,
}

impl PreparedAtomic {
    /// Prepare a fallible result before committing bytes. The callback performs
    /// no JavaScript, can run again on contention, and charges each retry.
    pub(crate) fn update<T, E>(
        &self,
        mut prepare: impl FnMut(u64) -> Result<(Option<u64>, T), E>,
    ) -> Result<T, E> {
        let word = &self.page.words[self.word];
        let mut observed = word.load(Ordering::SeqCst);
        loop {
            let old = (observed >> self.shift) & self.mask;
            #[cfg(not(target_has_atomic = "64"))]
            let old = u64::from(old);
            let (replacement, result) = prepare(old)?;
            let Some(replacement) = replacement else {
                return Ok(result);
            };
            let next = (observed & !(self.mask << self.shift))
                | (((replacement as Bits) & self.mask) << self.shift);
            match word.compare_exchange_weak(observed, next, Ordering::SeqCst, Ordering::SeqCst) {
                Ok(_) => return Ok(result),
                Err(current) => observed = current,
            }
        }
    }
}
