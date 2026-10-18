//! ArrayBuffer internal slots and zero-filled byte storage (25.1).
//!
//! Zero pages are implicit. This avoids an enormous eager allocation and zero-fill
//! merely to construct or grow a buffer, which can terminate an overcommitting
//! host even after `Vec::try_reserve` succeeds. Stored pages and their sorted index
//! use fallible allocation; logical byte offsets stay in the ToIndex range.

use super::{Budget, Error, Objects};
use spite_heap::Handle;
use std::fmt;

const MAX_BYTE_LENGTH: u64 = 9_007_199_254_740_991;
const PAGE_SIZE: usize = 4096;
const PAGE_SIZE_U64: u64 = 4096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArrayBufferError {
    Storage(Error),
    Detached,
    NotResizable,
    OutOfBounds,
    SameBuffer,
    Capacity,
}

impl fmt::Display for ArrayBufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => error.fmt(f),
            Self::Detached => f.write_str("ArrayBuffer is detached"),
            Self::NotResizable => f.write_str("ArrayBuffer is not resizable"),
            Self::OutOfBounds => f.write_str("ArrayBuffer range is out of bounds"),
            Self::SameBuffer => f.write_str("ArrayBuffer copy requires distinct buffers"),
            Self::Capacity => f.write_str("ArrayBuffer backing allocation failed"),
        }
    }
}

impl std::error::Error for ArrayBufferError {}

impl From<Error> for ArrayBufferError {
    fn from(error: Error) -> Self {
        Self::Storage(error)
    }
}

#[derive(Debug)]
struct Page {
    index: u64,
    bytes: Vec<u8>,
}

impl Page {
    fn new(index: u64) -> Result<Self, ArrayBufferError> {
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(PAGE_SIZE)
            .map_err(|_| ArrayBufferError::Capacity)?;
        // The capacity is already reserved; initializing one small page performs
        // no further allocation.
        bytes.resize(PAGE_SIZE, 0);
        Ok(Self { index, bytes })
    }
}

#[derive(Debug, Default)]
struct ByteDataBlock {
    pages: Vec<Page>,
}

impl ByteDataBlock {
    fn reserve_pages(
        &mut self,
        additional: usize,
        budget: &mut Budget,
    ) -> Result<(), ArrayBufferError> {
        if additional > self.pages.capacity() - self.pages.len() {
            // Growing the page index can move every existing record.
            budget.charge(self.pages.len())?;
        }
        self.pages
            .try_reserve(additional)
            .map_err(|_| ArrayBufferError::Capacity)
    }

    fn try_clone(&self, budget: &mut Budget) -> Result<Self, ArrayBufferError> {
        let mut copy = Self::default();
        budget.charge(self.pages.len())?;
        copy.reserve_pages(self.pages.len(), budget)?;
        for page in &self.pages {
            budget.charge(PAGE_SIZE)?;
            let mut new_page = Page::new(page.index)?;
            new_page.bytes.copy_from_slice(&page.bytes);
            copy.pages.push(new_page);
        }
        Ok(copy)
    }

    fn write(
        &mut self,
        mut start: u64,
        mut bytes: &[u8],
        budget: &mut Budget,
    ) -> Result<(), ArrayBufferError> {
        while !bytes.is_empty() {
            let index = start / PAGE_SIZE_U64;
            let offset = page_offset(start);
            let count = (PAGE_SIZE - offset).min(bytes.len());
            let (fragment, rest) = bytes.split_at(count);
            budget.charge(count)?;
            match self.pages.binary_search_by_key(&index, |page| page.index) {
                Ok(position) => {
                    self.pages[position].bytes[offset..offset + count].copy_from_slice(fragment);
                }
                Err(position) if fragment.iter().any(|byte| *byte != 0) => {
                    budget.charge(PAGE_SIZE)?;
                    budget.charge(self.pages.len() - position)?;
                    let mut page = Page::new(index)?;
                    page.bytes[offset..offset + count].copy_from_slice(fragment);
                    self.reserve_pages(1, budget)?;
                    self.pages.insert(position, page);
                }
                Err(_) => {}
            }
            start = start
                .checked_add(u64::try_from(count).map_err(|_| ArrayBufferError::OutOfBounds)?)
                .ok_or(ArrayBufferError::OutOfBounds)?;
            bytes = rest;
        }
        Ok(())
    }

    fn clear_range(
        &mut self,
        start: u64,
        end: u64,
        budget: &mut Budget,
    ) -> Result<(), ArrayBufferError> {
        for page in &mut self.pages {
            budget.charge(1)?;
            let page_start = page
                .index
                .checked_mul(PAGE_SIZE_U64)
                .ok_or(ArrayBufferError::OutOfBounds)?;
            let page_end = page_start
                .checked_add(PAGE_SIZE_U64)
                .ok_or(ArrayBufferError::OutOfBounds)?;
            let first = start.max(page_start);
            let last = end.min(page_end);
            if first < last {
                let first = page_offset(first);
                let count = usize::try_from(last - page_start)
                    .map_err(|_| ArrayBufferError::OutOfBounds)?;
                budget.charge(count - first)?;
                page.bytes[first..count].fill(0);
            }
        }
        Ok(())
    }

    fn copy_range(
        &self,
        start: u64,
        target: &mut Self,
        target_start: u64,
        count: u64,
        budget: &mut Budget,
    ) -> Result<(), ArrayBufferError> {
        let end = start
            .checked_add(count)
            .ok_or(ArrayBufferError::OutOfBounds)?;
        let target_end = target_start
            .checked_add(count)
            .ok_or(ArrayBufferError::OutOfBounds)?;
        target.clear_range(target_start, target_end, budget)?;
        for page in &self.pages {
            budget.charge(1)?;
            let page_start = page
                .index
                .checked_mul(PAGE_SIZE_U64)
                .ok_or(ArrayBufferError::OutOfBounds)?;
            let page_end = page_start
                .checked_add(PAGE_SIZE_U64)
                .ok_or(ArrayBufferError::OutOfBounds)?;
            let first = start.max(page_start);
            let last = end.min(page_end);
            if first < last {
                let offset = page_offset(first);
                let length =
                    usize::try_from(last - first).map_err(|_| ArrayBufferError::OutOfBounds)?;
                let destination = target_start
                    .checked_add(first - start)
                    .ok_or(ArrayBufferError::OutOfBounds)?;
                target.write(destination, &page.bytes[offset..offset + length], budget)?;
            }
        }
        Ok(())
    }
}

fn page_offset(index: u64) -> usize {
    usize::try_from(index % PAGE_SIZE_U64).expect("a page offset fits usize")
}

/// ArrayBuffer's data, byte length, and optional maximum-length internal slots.
/// The detach key is always undefined; no embedding API supplies another key.
/// Bytes contain no heap handles, so this slot introduces no traced edges.
#[derive(Debug)]
pub(crate) struct ArrayBufferData {
    block: Option<ByteDataBlock>,
    byte_length: u64,
    max_byte_length: Option<u64>,
}

impl ArrayBufferData {
    fn new(byte_length: u64, max_byte_length: Option<u64>) -> Result<Self, ArrayBufferError> {
        if byte_length > MAX_BYTE_LENGTH
            || max_byte_length.is_some_and(|max| max > MAX_BYTE_LENGTH || byte_length > max)
        {
            return Err(ArrayBufferError::OutOfBounds);
        }
        Ok(Self {
            block: Some(ByteDataBlock::default()),
            byte_length,
            max_byte_length,
        })
    }

    pub(crate) fn byte_length(&self) -> u64 {
        self.byte_length
    }

    pub(crate) fn max_byte_length(&self) -> u64 {
        if self.is_detached() {
            0
        } else {
            self.max_byte_length.unwrap_or(self.byte_length)
        }
    }

    pub(crate) fn is_detached(&self) -> bool {
        self.block.is_none()
    }

    pub(crate) fn is_resizable(&self) -> bool {
        self.max_byte_length.is_some()
    }

    fn block(&self) -> Result<&ByteDataBlock, ArrayBufferError> {
        self.block.as_ref().ok_or(ArrayBufferError::Detached)
    }

    fn check_range(&self, start: u64, count: u64) -> Result<(), ArrayBufferError> {
        self.block()?;
        if start
            .checked_add(count)
            .is_none_or(|end| end > self.byte_length)
        {
            return Err(ArrayBufferError::OutOfBounds);
        }
        Ok(())
    }
}

impl Objects {
    pub(crate) fn create_array_buffer(
        &mut self,
        prototype: &Handle,
        byte_length: u64,
        max_byte_length: Option<u64>,
        budget: &mut Budget,
    ) -> Result<Handle, ArrayBufferError> {
        let data = ArrayBufferData::new(byte_length, max_byte_length)?;
        budget.charge(1)?;
        let buffer = self.create(Some(prototype))?;
        self.object_mut(&buffer)?.array_buffer = Some(data);
        Ok(buffer)
    }

    fn array_buffer(&self, buffer: &Handle) -> Result<&ArrayBufferData, ArrayBufferError> {
        self.inspect(buffer)?
            .array_buffer_data()
            .ok_or(Error::WrongKind.into())
    }

    fn array_buffer_mut(
        &mut self,
        buffer: &Handle,
    ) -> Result<&mut ArrayBufferData, ArrayBufferError> {
        self.object_mut(buffer)?
            .array_buffer
            .as_mut()
            .ok_or(Error::WrongKind.into())
    }

    pub(crate) fn resize_array_buffer(
        &mut self,
        buffer: &Handle,
        new_length: u64,
        budget: &mut Budget,
    ) -> Result<(), ArrayBufferError> {
        let data = self.array_buffer(buffer)?;
        let max = data.max_byte_length.ok_or(ArrayBufferError::NotResizable)?;
        let block = data.block()?;
        if new_length > max {
            return Err(ArrayBufferError::OutOfBounds);
        }
        budget.charge(1)?;
        let retained = block
            .pages
            .partition_point(|page| page.index < new_length.div_ceil(PAGE_SIZE_U64));
        let has_partial_page = retained.checked_sub(1).is_some_and(|last| {
            new_length % PAGE_SIZE_U64 != 0 && block.pages[last].index == new_length / PAGE_SIZE_U64
        });
        budget.charge(block.pages.len() - retained)?;
        if has_partial_page {
            budget.charge(PAGE_SIZE - page_offset(new_length))?;
        }
        let data = self.array_buffer_mut(buffer)?;
        let block = data.block.as_mut().expect("validated attached buffer");
        block.pages.truncate(retained);
        // Truncation must erase bytes in the partially retained page, too: they
        // cannot reappear after subsequent growth (25.1.6.6).
        if has_partial_page {
            let page = block.pages.last_mut().expect("retained page");
            page.bytes[page_offset(new_length)..].fill(0);
        }
        data.byte_length = new_length;
        Ok(())
    }

    pub(crate) fn copy_array_buffer_bytes(
        &mut self,
        source: &Handle,
        source_start: u64,
        target: &Handle,
        target_start: u64,
        count: u64,
        budget: &mut Budget,
    ) -> Result<(), ArrayBufferError> {
        let source_data = self.array_buffer(source)?;
        source_data.check_range(source_start, count)?;
        let target_data = self.array_buffer(target)?;
        target_data.check_range(target_start, count)?;
        if source == target {
            return Err(ArrayBufferError::SameBuffer);
        }
        budget.charge(1)?;
        if count == 0 {
            return Ok(());
        }
        // All fallible work happens on a temporary block. Allocation or work
        // failure leaves both existing buffers unchanged.
        let mut copy = target_data.block()?.try_clone(budget)?;
        source_data
            .block()?
            .copy_range(source_start, &mut copy, target_start, count, budget)?;
        self.array_buffer_mut(target)?.block = Some(copy);
        Ok(())
    }

    /// Reads a single numeric element without allocating a temporary byte vector.
    pub(crate) fn read_array_buffer_bytes(
        &self,
        buffer: &Handle,
        index: u64,
        count: usize,
        budget: &mut Budget,
    ) -> Result<[u8; 8], ArrayBufferError> {
        if count > 8 {
            return Err(ArrayBufferError::OutOfBounds);
        }
        let data = self.array_buffer(buffer)?;
        data.check_range(
            index,
            u64::try_from(count).map_err(|_| ArrayBufferError::OutOfBounds)?,
        )?;
        let block = data.block()?;
        let search_work = block
            .pages
            .len()
            .checked_ilog2()
            .map_or(1, |log| log as usize + 2);
        budget.charge(1 + count * search_work)?;
        let mut result = [0; 8];
        for (offset, byte) in result.iter_mut().take(count).enumerate() {
            let position = index
                .checked_add(u64::try_from(offset).map_err(|_| ArrayBufferError::OutOfBounds)?)
                .ok_or(ArrayBufferError::OutOfBounds)?;
            if let Ok(page) = block
                .pages
                .binary_search_by_key(&(position / PAGE_SIZE_U64), |page| page.index)
            {
                *byte = block.pages[page].bytes[page_offset(position)];
            }
        }
        Ok(result)
    }

    /// Writes one numeric element atomically, touching at most two sparse pages.
    pub(crate) fn write_array_buffer_bytes(
        &mut self,
        buffer: &Handle,
        index: u64,
        bytes: &[u8],
        budget: &mut Budget,
    ) -> Result<(), ArrayBufferError> {
        if bytes.len() > 8 {
            return Err(ArrayBufferError::OutOfBounds);
        }
        let data = self.array_buffer_mut(buffer)?;
        data.check_range(
            index,
            u64::try_from(bytes.len()).map_err(|_| ArrayBufferError::OutOfBounds)?,
        )?;
        let block = data.block.as_mut().expect("validated attached buffer");
        let search_work = block
            .pages
            .len()
            .checked_ilog2()
            .map_or(1, |log| log as usize + 2);
        // Include the bounded preflight, insertion, and final byte lookups.
        budget.charge(1 + (bytes.len() + 4) * search_work)?;
        let mut staged: [Option<Page>; 2] = [None, None];
        let mut additional = 0;
        let mut remaining = bytes;
        let mut current = index;
        for staged_page in &mut staged {
            if remaining.is_empty() {
                break;
            }
            let page_index = current / PAGE_SIZE_U64;
            let count = (PAGE_SIZE - page_offset(current)).min(remaining.len());
            let (fragment, rest) = remaining.split_at(count);
            if let Err(position) = block
                .pages
                .binary_search_by_key(&page_index, |page| page.index)
            {
                if fragment.iter().any(|byte| *byte != 0) {
                    budget.charge(PAGE_SIZE)?;
                    budget.charge(
                        (block.pages.len() - position)
                            .checked_add(additional)
                            .ok_or(Error::WorkLimit)?,
                    )?;
                    *staged_page = Some(Page::new(page_index)?);
                    additional += 1;
                }
            }
            current = current
                .checked_add(u64::try_from(count).map_err(|_| ArrayBufferError::OutOfBounds)?)
                .ok_or(ArrayBufferError::OutOfBounds)?;
            remaining = rest;
        }
        // A failed reservation preserves the existing index and bytes. No work
        // charges, allocation, or other fallible steps follow this reservation.
        block.reserve_pages(additional, budget)?;
        for page in staged.into_iter().flatten() {
            let position = block
                .pages
                .binary_search_by_key(&page.index, |existing| existing.index)
                .expect_err("staged pages have distinct absent indices");
            block.pages.insert(position, page);
        }
        for (offset, byte) in bytes.iter().enumerate() {
            // check_range proved every numeric element position is below the
            // buffer length, and the length is at most MAX_BYTE_LENGTH.
            let position = index
                .checked_add(u64::try_from(offset).expect("a numeric element offset fits u64"))
                .expect("validated numeric element range");
            if let Ok(page) = block
                .pages
                .binary_search_by_key(&(position / PAGE_SIZE_U64), |page| page.index)
            {
                block.pages[page].bytes[page_offset(position)] = *byte;
            }
        }
        Ok(())
    }

    pub(crate) fn detach_array_buffer(
        &mut self,
        buffer: &Handle,
        budget: &mut Budget,
    ) -> Result<(), ArrayBufferError> {
        let data = self.array_buffer(buffer)?;
        budget.charge(data.block.as_ref().map_or(0, |block| block.pages.len()))?;
        let data = self.array_buffer_mut(buffer)?;
        data.block = None;
        data.byte_length = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unlimited() -> Budget {
        Budget::with_work_limit(None)
    }

    fn fixture(length: u64, max: Option<u64>) -> (Objects, Handle, Handle) {
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let buffer = objects
            .create_array_buffer(&prototype, length, max, &mut unlimited())
            .unwrap();
        (objects, prototype, buffer)
    }

    fn write(objects: &mut Objects, buffer: &Handle, start: u64, bytes: &[u8]) {
        let data = objects.array_buffer_mut(buffer).unwrap();
        data.check_range(start, u64::try_from(bytes.len()).unwrap())
            .unwrap();
        data.block
            .as_mut()
            .unwrap()
            .write(start, bytes, &mut unlimited())
            .unwrap();
    }

    fn read(objects: &Objects, buffer: &Handle, start: u64, count: usize) -> Vec<u8> {
        let data = objects.array_buffer(buffer).unwrap();
        data.check_range(start, u64::try_from(count).unwrap())
            .unwrap();
        (0..count)
            .map(|offset| {
                let index = start + u64::try_from(offset).unwrap();
                let block = data.block().unwrap();
                match block
                    .pages
                    .binary_search_by_key(&(index / PAGE_SIZE_U64), |page| page.index)
                {
                    Ok(position) => block.pages[position].bytes[page_offset(index)],
                    Err(_) => 0,
                }
            })
            .collect()
    }

    #[test]
    fn numeric_byte_io_crosses_pages_and_observes_resize_zeroing() {
        let (mut objects, _, buffer) = fixture(12_300, Some(20_000));
        let bytes = [1, 2, 3, 4, 5, 6, 7, 8];
        objects
            .write_array_buffer_bytes(&buffer, 4093, &bytes, &mut unlimited())
            .unwrap();
        assert_eq!(
            objects
                .read_array_buffer_bytes(&buffer, 4093, 8, &mut unlimited())
                .unwrap(),
            bytes
        );
        assert_eq!(
            read(&objects, &buffer, 4091, 12),
            [0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 0]
        );
        objects
            .write_array_buffer_bytes(&buffer, 8190, &[0; 8], &mut unlimited())
            .unwrap();
        assert_eq!(
            objects
                .array_buffer(&buffer)
                .unwrap()
                .block()
                .unwrap()
                .pages
                .len(),
            2
        );
        objects
            .resize_array_buffer(&buffer, 4097, &mut unlimited())
            .unwrap();
        assert_eq!(
            objects.read_array_buffer_bytes(&buffer, 4093, 8, &mut unlimited()),
            Err(ArrayBufferError::OutOfBounds)
        );
        objects
            .resize_array_buffer(&buffer, 12_300, &mut unlimited())
            .unwrap();
        assert_eq!(
            objects
                .read_array_buffer_bytes(&buffer, 4093, 8, &mut unlimited())
                .unwrap(),
            [1, 2, 3, 4, 0, 0, 0, 0]
        );
        objects
            .write_array_buffer_bytes(&buffer, 4093, &[0; 8], &mut unlimited())
            .unwrap();
        assert_eq!(
            objects
                .read_array_buffer_bytes(&buffer, 4093, 8, &mut unlimited())
                .unwrap(),
            [0; 8]
        );
        assert_eq!(
            objects.read_array_buffer_bytes(&buffer, 0, 9, &mut unlimited()),
            Err(ArrayBufferError::OutOfBounds)
        );
        assert_eq!(
            objects.write_array_buffer_bytes(&buffer, 0, &[1; 9], &mut unlimited()),
            Err(ArrayBufferError::OutOfBounds)
        );
        objects
            .detach_array_buffer(&buffer, &mut unlimited())
            .unwrap();
        assert_eq!(
            objects.write_array_buffer_bytes(&buffer, 0, &[1], &mut unlimited()),
            Err(ArrayBufferError::Detached)
        );
        assert_eq!(
            objects.read_array_buffer_bytes(&buffer, 0, 1, &mut unlimited()),
            Err(ArrayBufferError::Detached)
        );
    }

    #[test]
    fn numeric_writes_preflight_every_page_before_mutating_existing_storage() {
        let (mut objects, _, buffer) = fixture(12_300, None);
        objects
            .write_array_buffer_bytes(&buffer, 8192, &[9, 10], &mut unlimited())
            .unwrap();
        // The first absent page is staged before this budget fails on the
        // second. Neither the index nor any bytes may be committed early.
        assert_eq!(
            objects.write_array_buffer_bytes(
                &buffer,
                4093,
                &[1; 8],
                &mut Budget::new(PAGE_SIZE + 100)
            ),
            Err(ArrayBufferError::Storage(Error::WorkLimit))
        );
        assert_eq!(
            objects
                .read_array_buffer_bytes(&buffer, 4093, 8, &mut unlimited())
                .unwrap(),
            [0; 8]
        );
        assert_eq!(
            objects
                .read_array_buffer_bytes(&buffer, 8192, 2, &mut unlimited())
                .unwrap(),
            [9, 10, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(
            objects
                .array_buffer(&buffer)
                .unwrap()
                .block()
                .unwrap()
                .pages
                .len(),
            1
        );
        objects
            .write_array_buffer_bytes(&buffer, 4093, &[2; 3], &mut unlimited())
            .unwrap();
        // An existing first page must stay intact if the missing second page
        // cannot be prepared within the host's explicitly configured budget.
        assert_eq!(
            objects.write_array_buffer_bytes(&buffer, 4093, &[3; 8], &mut Budget::new(100)),
            Err(ArrayBufferError::Storage(Error::WorkLimit))
        );
        assert_eq!(
            objects
                .read_array_buffer_bytes(&buffer, 4093, 8, &mut unlimited())
                .unwrap(),
            [2, 2, 2, 0, 0, 0, 0, 0]
        );
        assert_eq!(
            objects.write_array_buffer_bytes(&buffer, u64::MAX, &[1], &mut unlimited()),
            Err(ArrayBufferError::OutOfBounds)
        );
        // Updating an existing page needs only bounded element/lookup work,
        // without copying the other stored pages under this small opt-in budget.
        objects
            .write_array_buffer_bytes(&buffer, 8192, &[12], &mut Budget::new(50))
            .unwrap();
        assert_eq!(
            objects
                .read_array_buffer_bytes(&buffer, 8192, 2, &mut unlimited())
                .unwrap(),
            [12, 10, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn numeric_byte_io_keeps_safe_integer_offsets_without_narrowing() {
        let (mut objects, _, buffer) = fixture(MAX_BYTE_LENGTH, None);
        let bytes = [8, 7, 6, 5, 4, 3, 2, 1];
        objects
            .write_array_buffer_bytes(&buffer, MAX_BYTE_LENGTH - 8, &bytes, &mut unlimited())
            .unwrap();
        assert_eq!(
            objects
                .read_array_buffer_bytes(&buffer, MAX_BYTE_LENGTH - 8, 8, &mut unlimited())
                .unwrap(),
            bytes
        );
        assert_eq!(
            objects.read_array_buffer_bytes(&buffer, MAX_BYTE_LENGTH - 7, 8, &mut unlimited()),
            Err(ArrayBufferError::OutOfBounds)
        );
        assert_eq!(
            objects.write_array_buffer_bytes(&buffer, MAX_BYTE_LENGTH, &[1], &mut unlimited()),
            Err(ArrayBufferError::OutOfBounds)
        );
        assert_eq!(
            objects
                .read_array_buffer_bytes(&buffer, MAX_BYTE_LENGTH, 0, &mut unlimited())
                .unwrap(),
            [0; 8]
        );
    }

    #[test]
    fn sparse_storage_preserves_bytes_across_pages_and_zeroes_shrink_regrowth() {
        let (mut objects, _, buffer) = fixture(12_300, Some(20_000));
        write(&mut objects, &buffer, 4094, &[1, 2, 3, 4, 5]);
        write(&mut objects, &buffer, 12_288, &[6, 7, 8]);
        assert_eq!(
            read(&objects, &buffer, 4092, 9),
            [0, 0, 1, 2, 3, 4, 5, 0, 0]
        );
        objects
            .resize_array_buffer(&buffer, 4097, &mut unlimited())
            .unwrap();
        objects
            .resize_array_buffer(&buffer, 12_300, &mut unlimited())
            .unwrap();
        assert_eq!(read(&objects, &buffer, 4094, 5), [1, 2, 3, 0, 0]);
        assert_eq!(read(&objects, &buffer, 12_288, 3), [0, 0, 0]);
        objects
            .resize_array_buffer(&buffer, 4096, &mut unlimited())
            .unwrap();
        objects
            .resize_array_buffer(&buffer, 12_300, &mut unlimited())
            .unwrap();
        assert_eq!(read(&objects, &buffer, 4094, 5), [1, 2, 0, 0, 0]);
        objects
            .resize_array_buffer(&buffer, 0, &mut unlimited())
            .unwrap();
        objects
            .resize_array_buffer(&buffer, 12_300, &mut unlimited())
            .unwrap();
        assert_eq!(read(&objects, &buffer, 4094, 5), [0, 0, 0, 0, 0]);
    }

    #[test]
    fn range_copy_overwrites_holes_and_preserves_unaffected_target_bytes() {
        let (mut objects, prototype, source) = fixture(16_384, None);
        let target = objects
            .create_array_buffer(&prototype, 20_000, None, &mut unlimited())
            .unwrap();
        write(&mut objects, &source, 4094, &[1, 2, 3, 4, 5]);
        write(&mut objects, &target, 0, &[9, 9, 9]);
        write(&mut objects, &target, 4095, &[8, 8, 8, 8]);
        write(&mut objects, &target, 16_384, &[7, 7, 7]);
        objects
            .copy_array_buffer_bytes(&source, 0, &target, 1, 16_383, &mut unlimited())
            .unwrap();
        assert_eq!(read(&objects, &target, 0, 4), [9, 0, 0, 0]);
        assert_eq!(read(&objects, &target, 4094, 7), [0, 1, 2, 3, 4, 5, 0]);
        assert_eq!(read(&objects, &target, 16_382, 5), [0, 0, 7, 7, 7]);
        assert_eq!(read(&objects, &source, 4094, 5), [1, 2, 3, 4, 5]);
        objects
            .copy_array_buffer_bytes(&source, 4094, &target, 8190, 5, &mut unlimited())
            .unwrap();
        assert_eq!(read(&objects, &target, 8189, 7), [0, 1, 2, 3, 4, 5, 0]);
    }

    #[test]
    fn invalid_states_and_work_failures_leave_buffers_unchanged() {
        let (mut objects, prototype, source) = fixture(8192, Some(16_384));
        let target = objects
            .create_array_buffer(&prototype, 8192, None, &mut unlimited())
            .unwrap();
        write(&mut objects, &source, 4095, &[1, 2, 3]);
        write(&mut objects, &target, 4095, &[7, 8, 9]);
        assert_eq!(
            objects.resize_array_buffer(&source, 20_000, &mut unlimited()),
            Err(ArrayBufferError::OutOfBounds)
        );
        assert_eq!(
            objects.resize_array_buffer(&source, 4096, &mut Budget::new(1)),
            Err(ArrayBufferError::Storage(Error::WorkLimit))
        );
        assert_eq!(
            objects.copy_array_buffer_bytes(
                &source,
                0,
                &target,
                0,
                8192,
                &mut Budget::new(PAGE_SIZE + 2)
            ),
            Err(ArrayBufferError::Storage(Error::WorkLimit))
        );
        assert_eq!(
            objects.copy_array_buffer_bytes(&source, u64::MAX, &target, 0, 1, &mut unlimited()),
            Err(ArrayBufferError::OutOfBounds)
        );
        assert_eq!(
            objects.copy_array_buffer_bytes(&source, 0, &source, 0, 0, &mut unlimited()),
            Err(ArrayBufferError::SameBuffer)
        );
        assert_eq!(
            objects.resize_array_buffer(&target, 0, &mut unlimited()),
            Err(ArrayBufferError::NotResizable)
        );
        assert_eq!(read(&objects, &source, 4095, 3), [1, 2, 3]);
        assert_eq!(read(&objects, &target, 4095, 3), [7, 8, 9]);
        assert_eq!(objects.array_buffer(&source).unwrap().byte_length(), 8192);
        let data = objects.array_buffer_mut(&target).unwrap();
        assert_eq!(
            data.block
                .as_mut()
                .unwrap()
                .reserve_pages(usize::MAX, &mut unlimited()),
            Err(ArrayBufferError::Capacity)
        );
        assert_eq!(read(&objects, &target, 4095, 3), [7, 8, 9]);
    }

    #[test]
    fn transfer_storage_detaches_only_after_success_and_retains_resizability() {
        let (mut objects, prototype, source) = fixture(8, Some(32));
        write(&mut objects, &source, 1, &[1, 2, 3]);
        let target = objects
            .create_array_buffer(&prototype, 16, Some(32), &mut unlimited())
            .unwrap();
        objects
            .copy_array_buffer_bytes(&source, 0, &target, 0, 8, &mut unlimited())
            .unwrap();
        objects
            .detach_array_buffer(&source, &mut unlimited())
            .unwrap();
        let data = objects.array_buffer(&source).unwrap();
        assert!(data.is_detached());
        assert!(data.is_resizable());
        assert_eq!(data.byte_length(), 0);
        assert_eq!(data.max_byte_length(), 0);
        assert_eq!(read(&objects, &target, 0, 6), [0, 1, 2, 3, 0, 0]);
        assert_eq!(read(&objects, &target, 8, 8), [0; 8]);
        assert_eq!(
            objects.resize_array_buffer(&source, 0, &mut unlimited()),
            Err(ArrayBufferError::Detached)
        );
        assert_eq!(
            objects.copy_array_buffer_bytes(&source, 0, &target, 0, 0, &mut unlimited()),
            Err(ArrayBufferError::Detached)
        );
    }

    #[test]
    fn safe_integer_lengths_need_no_eager_pages_and_allocation_failure_preserves_source() {
        let (mut objects, prototype, source) = fixture(MAX_BYTE_LENGTH, Some(MAX_BYTE_LENGTH));
        write(&mut objects, &source, MAX_BYTE_LENGTH - 1, &[13]);
        assert_eq!(read(&objects, &source, MAX_BYTE_LENGTH - 2, 2), [0, 13]);
        let target = objects
            .create_array_buffer(&prototype, MAX_BYTE_LENGTH, None, &mut unlimited())
            .unwrap();
        objects
            .copy_array_buffer_bytes(&source, 0, &target, 0, MAX_BYTE_LENGTH, &mut unlimited())
            .unwrap();
        assert_eq!(read(&objects, &target, MAX_BYTE_LENGTH - 2, 2), [0, 13]);
        objects
            .copy_array_buffer_bytes(
                &source,
                MAX_BYTE_LENGTH,
                &target,
                MAX_BYTE_LENGTH,
                0,
                &mut unlimited(),
            )
            .unwrap();
        assert_eq!(
            objects.create_array_buffer(&prototype, MAX_BYTE_LENGTH + 1, None, &mut unlimited()),
            Err(ArrayBufferError::OutOfBounds)
        );
        assert_eq!(
            objects.create_array_buffer(&prototype, 1, Some(0), &mut unlimited()),
            Err(ArrayBufferError::OutOfBounds)
        );
        let mut limited = Objects::with_limits(Some(2), None);
        let prototype = limited.create(None).unwrap();
        let buffer = limited
            .create_array_buffer(&prototype, 3, None, &mut unlimited())
            .unwrap();
        write(&mut limited, &buffer, 0, &[1, 2, 3]);
        assert_eq!(
            limited.create_array_buffer(&prototype, 3, None, &mut unlimited()),
            Err(ArrayBufferError::Storage(Error::Heap(
                spite_heap::Error::Capacity
            )))
        );
        assert_eq!(read(&limited, &buffer, 0, 3), [1, 2, 3]);
        assert!(!limited.array_buffer(&buffer).unwrap().is_detached());
    }
}
