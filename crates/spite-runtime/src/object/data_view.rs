//! DataView internal slots and live buffer witnesses (25.3.1).

use super::{ArrayBufferData, Error, Objects};
use spite_heap::Handle;

#[derive(Debug)]
pub(crate) struct DataViewData {
    buffer: Handle,
    byte_offset: u64,
    // None is the specification's auto byte length, not a zero-length view.
    byte_length: Option<u64>,
}

impl DataViewData {
    pub(crate) fn buffer(&self) -> &Handle {
        &self.buffer
    }

    pub(crate) fn byte_offset(&self) -> u64 {
        self.byte_offset
    }

    pub(crate) fn byte_length(&self) -> Option<u64> {
        self.byte_length
    }

    /// IsViewOutOfBounds and GetViewByteLength using a single live buffer state.
    /// Original slots remain intact so a view can recover after buffer growth.
    pub(crate) fn in_bounds_byte_length(&self, buffer: &ArrayBufferData) -> Option<u64> {
        if buffer.is_detached() {
            return None;
        }
        let available = buffer.byte_length().checked_sub(self.byte_offset)?;
        match self.byte_length() {
            Some(length) if length <= available => Some(length),
            Some(_) => None,
            None => Some(available),
        }
    }
}

impl Objects {
    pub(crate) fn create_data_view(
        &mut self,
        prototype: &Handle,
        buffer: &Handle,
        byte_offset: u64,
        byte_length: Option<u64>,
    ) -> Result<Handle, Error> {
        self.inspect(prototype)?;
        let buffer_data = self
            .inspect(buffer)?
            .array_buffer_data()
            .ok_or(Error::WrongKind)?;
        let data = DataViewData {
            buffer: buffer.clone(),
            byte_offset,
            byte_length,
        };
        // JavaScript performs its observable checks before calling storage. Keep
        // malformed internal records and foreign/stale edges out of the heap.
        if (byte_length.is_none() && !buffer_data.is_resizable())
            || data.in_bounds_byte_length(buffer_data).is_none()
        {
            return Err(Error::WrongKind);
        }
        let view = self.create(Some(prototype))?;
        self.object_mut(&view)?.data_view = Some(data);
        Ok(view)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::{ArrayBufferError, Budget};

    fn unlimited() -> Budget {
        Budget::with_work_limit(None)
    }

    fn length(objects: &Objects, view: &Handle) -> Option<u64> {
        let data = objects.inspect(view).unwrap().data_view_data().unwrap();
        let buffer = objects
            .inspect(data.buffer())
            .unwrap()
            .array_buffer_data()
            .unwrap();
        data.in_bounds_byte_length(buffer)
    }

    #[test]
    fn fixed_and_auto_witnesses_recover_after_buffer_shrink_and_growth() {
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let buffer = objects
            .create_array_buffer(&prototype, 10, Some(20), &mut unlimited())
            .unwrap();
        let fixed = objects
            .create_data_view(&prototype, &buffer, 4, Some(6))
            .unwrap();
        let auto = objects
            .create_data_view(&prototype, &buffer, 4, None)
            .unwrap();
        assert_eq!(
            (length(&objects, &fixed), length(&objects, &auto)),
            (Some(6), Some(6))
        );
        objects
            .resize_array_buffer(&buffer, 8, &mut unlimited())
            .unwrap();
        assert_eq!(
            (length(&objects, &fixed), length(&objects, &auto)),
            (None, Some(4))
        );
        objects
            .resize_array_buffer(&buffer, 4, &mut unlimited())
            .unwrap();
        assert_eq!(
            (length(&objects, &fixed), length(&objects, &auto)),
            (None, Some(0))
        );
        objects
            .resize_array_buffer(&buffer, 3, &mut unlimited())
            .unwrap();
        assert_eq!(
            (length(&objects, &fixed), length(&objects, &auto)),
            (None, None)
        );
        objects
            .resize_array_buffer(&buffer, 12, &mut unlimited())
            .unwrap();
        assert_eq!(
            (length(&objects, &fixed), length(&objects, &auto)),
            (Some(6), Some(8))
        );
        let data = objects.inspect(&fixed).unwrap().data_view_data().unwrap();
        assert_eq!((data.byte_offset(), data.byte_length()), (4, Some(6)));
        objects
            .detach_array_buffer(&buffer, &mut unlimited())
            .unwrap();
        assert_eq!(
            (length(&objects, &fixed), length(&objects, &auto)),
            (None, None)
        );
        assert_eq!(
            objects
                .inspect(&fixed)
                .unwrap()
                .data_view_data()
                .unwrap()
                .buffer(),
            &buffer
        );
    }

    #[test]
    fn view_buffer_edges_are_traced_and_invalid_records_are_rejected_before_allocation() {
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let buffer = objects
            .create_array_buffer(&prototype, 8, None, &mut unlimited())
            .unwrap();
        let view = objects
            .create_data_view(&prototype, &buffer, 1, Some(7))
            .unwrap();
        let unrelated = objects.create(None).unwrap();
        let mut other = Objects::with_limits(None, None);
        let foreign = other.create(None).unwrap();
        for (proto, source) in [(&foreign, &buffer), (&prototype, &foreign)] {
            assert_eq!(
                objects.create_data_view(proto, source, 0, Some(0)),
                Err(Error::Heap(spite_heap::Error::ForeignHandle))
            );
        }
        assert_eq!(
            objects.create_data_view(&prototype, &unrelated, 0, Some(0)),
            Err(Error::WrongKind)
        );
        for (offset, length) in [(9, Some(0)), (0, Some(9)), (u64::MAX, Some(1)), (0, None)] {
            assert_eq!(
                objects.create_data_view(&prototype, &buffer, offset, length),
                Err(Error::WrongKind)
            );
        }
        let collection = objects.collect([&view], 100).unwrap();
        assert_eq!((collection.live, collection.reclaimed), (3, 1));
        assert!(objects.inspect(&buffer).is_ok());
        assert!(matches!(
            objects.inspect(&unrelated),
            Err(Error::Heap(spite_heap::Error::StaleHandle))
        ));
        objects
            .detach_array_buffer(&buffer, &mut unlimited())
            .unwrap();
        assert_eq!(
            objects.create_data_view(&prototype, &buffer, 0, Some(0)),
            Err(Error::WrongKind)
        );
        assert_eq!(
            objects.read_array_buffer_bytes(&buffer, 0, 0, &mut unlimited()),
            Err(ArrayBufferError::Detached)
        );
        assert_eq!(objects.collect([&prototype], 100).unwrap().reclaimed, 2);
        assert!(matches!(
            objects.inspect(&buffer),
            Err(Error::Heap(spite_heap::Error::StaleHandle))
        ));
    }
}
