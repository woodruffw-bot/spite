//! Atomics on ordinary and shared integer TypedArrays (25.4).

use super::{global_metadata::Member, typed_array_methods::TypedArrayWitness};
use crate::{Error, ExceptionKind, Realm, Value, binary_data::ElementType};
use spite_core::Span;

mod wait_queue;
pub(crate) use wait_queue::WaitRegistry;

impl Realm {
    pub(super) fn atomics_method(
        &mut self,
        method: Member,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        if matches!(method, Member::AtomicsIsLockFree) {
            let size = self.atomics_integer(arguments.next().unwrap_or(Value::Undefined), span)?;
            // Agent Record flags (9.7): use the target's native atomic widths.
            // Four bytes are required to be lock-free on every supported target.
            return Ok(Value::Boolean(
                size == 4.0
                    || (size == 1.0 && cfg!(target_has_atomic = "8"))
                    || (size == 2.0 && cfg!(target_has_atomic = "16"))
                    || (size == 8.0 && cfg!(target_has_atomic = "64")),
            ));
        }

        let waitable = matches!(
            method,
            Member::AtomicsWait | Member::AtomicsWaitAsync | Member::AtomicsNotify
        );
        let witness =
            self.atomics_validate(arguments.next().unwrap_or(Value::Undefined), waitable, span)?;
        let shared = self.object_work(span, |objects, _| {
            let data = objects
                .inspect(&witness.object)?
                .typed_array_data()
                .expect("validated TypedArray");
            Ok(objects
                .inspect(data.buffer())?
                .buffer_info()?
                .expect("typed buffer")
                .shared)
        })?;
        let waiting = matches!(method, Member::AtomicsWait | Member::AtomicsWaitAsync);
        if waiting && !shared {
            // DoWait (25.4.3.14) rejects ordinary buffers before converting
            // index, value, or timeout (DoWait, 25.4.3.14).
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Atomics waiting requires a SharedArrayBuffer",
            ));
        }
        let index =
            self.atomics_index(&witness, arguments.next().unwrap_or(Value::Undefined), span)?;
        if waiting {
            return self.atomics_wait(method, witness, index, arguments, span);
        }
        if matches!(method, Member::AtomicsNotify) {
            // Count conversion precedes the ordinary-buffer +0 return or shared
            // wait-list lookup, without post-conversion view revalidation.
            let count = arguments.next().unwrap_or(Value::Undefined);
            let count = if matches!(count, Value::Undefined) {
                f64::INFINITY
            } else {
                self.atomics_integer(count, span)?.max(0.0)
            };
            if !shared {
                return Ok(Value::Number(0.0));
            }
            let (backing, byte_index) = self.object_work(span, |objects, _| {
                let data = objects
                    .inspect(&witness.object)?
                    .typed_array_data()
                    .expect("waitable view");
                let backing = objects
                    .inspect(data.buffer())?
                    .shared_array_buffer_data()
                    .expect("shared waitable buffer")
                    .clone();
                Ok((
                    backing,
                    data.byte_offset() + index * witness.element.size() as u64,
                ))
            })?;
            let count = self.notify_atomic_waiters(&backing, byte_index, count as usize, span)?;
            return Ok(Value::Number(count as f64));
        }

        let value = if matches!(method, Member::AtomicsLoad) {
            None
        } else {
            Some(self.atomics_numeric(
                witness.element,
                arguments.next().unwrap_or(Value::Undefined),
                span,
            )?)
        };
        // compareExchange converts both operands even when the first conversion
        // detached or resized the buffer, and even when comparison will fail.
        let replacement = if matches!(method, Member::AtomicsCompareExchange) {
            Some(self.atomics_numeric(
                witness.element,
                arguments.next().unwrap_or(Value::Undefined),
                span,
            )?)
        } else {
            None
        };

        let (buffer, byte_index) = self.object_work(span, |objects, _| {
            let data = objects
                .inspect(&witness.object)?
                .typed_array_data()
                .expect("validated TypedArray");
            // The original witness bounds index * size + offset within the
            // safe buffer domain. These immutable layout slots cannot change.
            Ok((
                data.buffer().clone(),
                data.byte_offset() + index * witness.element.size() as u64,
            ))
        })?;
        // RevalidateAtomicAccess (25.4.3.4) after every observable conversion.
        // A fixed view that became out of bounds throws TypeError; a still-live
        // tracking view whose requested byte disappeared throws RangeError.
        self.typed_array_validate(Value::Object(witness.object), span)?;
        let byte_length = self.object_work(span, |objects, _| {
            Ok(objects
                .inspect(&buffer)?
                .buffer_info()?
                .expect("TypedArray buffer")
                .byte_length())
        })?;
        if byte_index >= byte_length {
            return Err(Self::atomics_range(span));
        }

        if shared {
            return self.atomics_shared(
                method,
                witness.element,
                &buffer,
                byte_index,
                value,
                replacement,
                span,
            );
        }

        let little_endian = cfg!(target_endian = "little");
        let size = witness.element.size();
        if matches!(method, Member::AtomicsStore) {
            let value = value.expect("converted store value");
            self.object_work(span, |_, budget| budget.value(&value))?;
            let bytes = self.integer_work(span, |budget| {
                witness.element.encode(value.clone(), little_endian, budget)
            })?;
            self.buffer_value_work(span, |objects, budget| {
                objects.write_array_buffer_bytes(&buffer, byte_index, &bytes[..size], budget)
            })?;
            return Ok(value);
        }
        let bytes = self.buffer_value_work(span, |objects, budget| {
            objects.read_array_buffer_bytes(&buffer, byte_index, size, budget)
        })?;
        // Decode the return value before writing, so an opted-in BigInt or
        // work failure cannot occur after the memory update.
        let old = self.integer_work(span, |budget| {
            witness.element.decode(bytes, little_endian, budget)
        })?;
        let Some(value) = value else {
            return Ok(old);
        };
        let operand = self.integer_work(span, |budget| {
            witness.element.encode(value, little_endian, budget)
        })?;
        let updated = match method {
            Member::AtomicsExchange => operand,
            Member::AtomicsCompareExchange => {
                let replacement = self.integer_work(span, |budget| {
                    witness.element.encode(
                        replacement.expect("converted replacement"),
                        little_endian,
                        budget,
                    )
                })?;
                if bytes != operand {
                    return Ok(old);
                }
                replacement
            }
            _ => {
                // Modular integer arithmetic on the encoded words is exact
                // for every Number and BigInt element type, including overflow.
                // Normalize only the active bytes on big-endian targets.
                let mut x = bytes;
                let mut y = operand;
                if !little_endian {
                    x[..size].reverse();
                    y[..size].reverse();
                }
                let x = u64::from_le_bytes(x);
                let y = u64::from_le_bytes(y);
                let bits = match method {
                    Member::AtomicsAdd => x.wrapping_add(y),
                    Member::AtomicsSub => x.wrapping_sub(y),
                    Member::AtomicsAnd => x & y,
                    Member::AtomicsOr => x | y,
                    Member::AtomicsXor => x ^ y,
                    _ => unreachable!("Atomics method dispatched above"),
                };
                let mut updated = bits.to_le_bytes();
                if !little_endian {
                    updated[..size].reverse();
                }
                updated
            }
        };
        // No JavaScript or concurrent agent can run between this ordinary
        // buffer read and write. Shared data blocks need a separate backend.
        self.buffer_value_work(span, |objects, budget| {
            objects.write_array_buffer_bytes(&buffer, byte_index, &updated[..size], budget)
        })?;
        Ok(old)
    }

    fn atomics_wait(
        &mut self,
        method: Member,
        witness: TypedArrayWitness,
        index: u64,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        // DoWait (25.4.3.14): value and timeout conversions precede the
        // AgentCanSuspend check, even for mismatches and zero timeouts.
        let value = self.atomics_numeric(
            witness.element,
            arguments.next().unwrap_or(Value::Undefined),
            span,
        )?;
        let little_endian = cfg!(target_endian = "little");
        let size = witness.element.size();
        let mut expected = self.integer_work(span, |budget| {
            witness.element.encode(value, little_endian, budget)
        })?;
        if !little_endian {
            expected[..size].reverse();
        }
        let expected = u64::from_le_bytes(expected);
        let timeout = self.number(arguments.next().unwrap_or(Value::Undefined), span)?;
        let timeout = if timeout.is_nan() {
            f64::INFINITY
        } else {
            timeout.max(0.0)
        };
        if matches!(method, Member::AtomicsWait) && !self.can_suspend {
            // This agent cannot suspend its execution thread.
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "this agent cannot suspend for Atomics.wait",
            ));
        }
        if size > crate::object::shared_page::WORD_BYTES {
            return Err(Self::unsupported(
                span,
                "shared atomics of this width require native target support",
            ));
        }
        let (backing, byte_index) = self.object_work(span, |objects, _| {
            let data = objects
                .inspect(&witness.object)?
                .typed_array_data()
                .expect("validated waitable view");
            let index = data.byte_offset() + index * size as u64;
            let backing = objects
                .inspect(data.buffer())?
                .shared_array_buffer_data()
                .expect("shared waitable buffer")
                .clone();
            Ok((backing, index))
        })?;
        let access = self.buffer_value_work(span, |_, budget| {
            backing.prepare_atomic(byte_index, size, budget)
        })?;
        if matches!(method, Member::AtomicsWait) {
            return self
                .blocking_atomic_wait(backing, byte_index, &access, expected, timeout, span);
        }
        // DoWait creates the intrinsic capability and result before its shared
        // memory read, including immediate results. Neither invokes public hooks.
        let capability = self.intrinsic_promise_capability(span)?;
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .object_prototype
            .clone();
        let result = self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
        let observed = access.update(|old| {
            self.tick(span)?;
            Ok((None, old))
        })?;
        let value = if observed != expected {
            "not-equal"
        } else if timeout == 0.0 {
            "timed-out"
        } else {
            self.atomic_wait_record(&result, true, capability.promise.clone(), span)?;
            if self.register_atomic_wait(
                backing, byte_index, &access, expected, capability, timeout, span,
            )? {
                return Ok(Value::Object(result));
            }
            "not-equal"
        };
        self.atomic_wait_record(
            &result,
            false,
            Value::String(spite_core::JsString::from(value)),
            span,
        )
    }

    fn atomic_wait_record(
        &mut self,
        result: &spite_heap::Handle,
        asynchronous: bool,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        self.object_work(span, |objects, budget| {
            for (key, value) in [("async", Value::Boolean(asynchronous)), ("value", value)] {
                objects.define(
                    result,
                    key,
                    crate::object::DataDescriptor {
                        value: Some(value),
                        writable: Some(true),
                        enumerable: Some(true),
                        configurable: Some(true),
                    },
                    budget,
                )?;
            }
            Ok(Value::Object(result.clone()))
        })
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "converted atomic operands and immutable layout"
    )]
    fn atomics_shared(
        &mut self,
        method: Member,
        element: ElementType,
        buffer: &spite_heap::Handle,
        index: u64,
        value: Option<Value>,
        replacement: Option<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let size = element.size();
        if size > crate::object::shared_page::WORD_BYTES {
            return Err(Self::unsupported(
                span,
                "shared atomics of this width require native target support",
            ));
        }
        let little_endian = cfg!(target_endian = "little");
        let bits = |mut bytes: [u8; 8]| {
            if !little_endian {
                bytes[..size].reverse();
            }
            u64::from_le_bytes(bytes)
        };
        let operand = value
            .as_ref()
            .map(|value| {
                self.integer_work(span, |budget| {
                    element.encode(value.clone(), little_endian, budget)
                })
            })
            .transpose()?
            .map(bits);
        let replacement = replacement
            .map(|value| {
                self.integer_work(span, |budget| element.encode(value, little_endian, budget))
            })
            .transpose()?
            .map(bits);
        let store = matches!(method, Member::AtomicsStore);
        if store {
            self.object_work(span, |_, budget| {
                budget.value(value.as_ref().expect("store value"))
            })?;
        }
        // Storage preparation can allocate or lock the sparse directory. It
        // returns an owning Arc and releases that lock before the atomic step
        // described by Atomics.isLockFree (25.4.11). No JS runs in either step.
        let access = self.buffer_value_work(span, |objects, budget| {
            objects
                .inspect(buffer)?
                .shared_array_buffer_data()
                .expect("shared buffer")
                .prepare_atomic(index, size, budget)
        })?;
        access.update(|old_bits| {
            self.tick(span)?;
            if store {
                return Ok((operand, value.clone().expect("store value")));
            }
            let mut bytes = old_bits.to_le_bytes();
            if !little_endian {
                bytes[..size].reverse();
            }
            // Prepare the fallible BigInt return before a successful update.
            let old =
                self.integer_work(span, |budget| element.decode(bytes, little_endian, budget))?;
            let next = operand.and_then(|operand| match method {
                Member::AtomicsAdd => Some(old_bits.wrapping_add(operand)),
                Member::AtomicsSub => Some(old_bits.wrapping_sub(operand)),
                Member::AtomicsAnd => Some(old_bits & operand),
                Member::AtomicsOr => Some(old_bits | operand),
                Member::AtomicsXor => Some(old_bits ^ operand),
                Member::AtomicsExchange => Some(operand),
                Member::AtomicsCompareExchange => {
                    (old_bits == operand).then_some(replacement.expect("replacement value"))
                }
                _ => unreachable!("atomic method"),
            });
            Ok((next, old))
        })
    }

    fn atomics_validate(
        &mut self,
        value: Value,
        waitable: bool,
        span: Span,
    ) -> Result<TypedArrayWitness, Error> {
        let witness = self.typed_array_validate(value, span)?;
        let valid = if waitable {
            matches!(witness.element, ElementType::Int32 | ElementType::BigInt64)
        } else {
            !matches!(
                witness.element,
                ElementType::Uint8Clamped
                    | ElementType::Float16
                    | ElementType::Float32
                    | ElementType::Float64
            )
        };
        if !valid {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "invalid TypedArray element type for Atomics operation",
            ));
        }
        Ok(witness)
    }

    fn atomics_integer(&mut self, value: Value, span: Span) -> Result<f64, Error> {
        let integer = self.number(value, span)?.trunc();
        Ok(if integer.is_nan() || integer == 0.0 {
            0.0
        } else {
            integer
        })
    }

    fn atomics_numeric(
        &mut self,
        element: ElementType,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        if element.is_bigint() {
            self.bigint(value, span).map(Value::BigInt)
        } else {
            self.atomics_integer(value, span).map(Value::Number)
        }
    }

    fn atomics_index(
        &mut self,
        witness: &TypedArrayWitness,
        value: Value,
        span: Span,
    ) -> Result<u64, Error> {
        // ToIndex (7.1.22) and ValidateAtomicAccess (25.4.3.2) use the
        // initially captured length, including when coercion grows the view.
        let index = self.atomics_integer(value, span)?;
        if index < 0.0 || index >= witness.length as f64 {
            return Err(Self::atomics_range(span));
        }
        Ok(index as u64)
    }

    fn atomics_range(span: Span) -> Error {
        Self::exception(
            ExceptionKind::RangeError,
            span,
            "Atomics index is outside the buffer range",
        )
    }
}
