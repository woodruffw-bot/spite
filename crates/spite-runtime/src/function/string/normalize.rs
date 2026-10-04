//! Unicode normalization forms, canonical ordering, and blocking (22.1.3.15, UAX #15).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{
    JsString, Span, canonical_combining_class, canonical_composition, hangul_decomposition,
    unicode_decomposition,
};

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug)]
pub(super) enum Form {
    Nfc,
    Nfd,
    Nfkc,
    Nfkd,
}

impl Form {
    fn compatibility(self) -> bool {
        matches!(self, Self::Nfkc | Self::Nfkd)
    }
    fn composition(self) -> bool {
        matches!(self, Self::Nfc | Self::Nfkc)
    }
}

#[derive(Clone, Copy, Default)]
struct Point {
    code: u32,
    class: u8,
}

impl Realm {
    pub(crate) fn string_normalize(
        &mut self,
        receiver: Value,
        form: Value,
        span: Span,
    ) -> Result<Value, Error> {
        Self::require_object_coercible(&receiver, span)?;
        let string = self.string(receiver, span)?;
        let form = if matches!(form, Value::Undefined) {
            Form::Nfc
        } else {
            let form = self.string(form, span)?;
            match form.code_units() {
                [0x4e, 0x46, 0x43] => Form::Nfc,
                [0x4e, 0x46, 0x44] => Form::Nfd,
                [0x4e, 0x46, 0x4b, 0x43] => Form::Nfkc,
                [0x4e, 0x46, 0x4b, 0x44] => Form::Nfkd,
                _ => {
                    return Err(Self::exception(
                        ExceptionKind::RangeError,
                        span,
                        "invalid normalization form",
                    ));
                }
            }
        };
        self.normalize_string(&string, form, span)
            .map(Value::String)
    }

    pub(super) fn normalize_string(
        &mut self,
        string: &JsString,
        form: Form,
        span: Span,
    ) -> Result<JsString, Error> {
        let mut points = Vec::new();
        let mut pending = Vec::new();
        for decoded in char::decode_utf16(string.code_units().iter().copied()) {
            let point = decoded.map_or_else(
                |error| u32::from(error.unpaired_surrogate()),
                |point| point as u32,
            );
            self.push_normalization_points(&mut pending, &[point], span)?;
            while let Some(point) = pending.pop() {
                self.tick(span)?;
                if let Some((mapping, length)) = hangul_decomposition(point) {
                    self.push_normalization_points(&mut pending, &mapping[..length], span)?;
                } else if let Some(mapping) = unicode_decomposition(point, form.compatibility()) {
                    self.push_normalization_points(&mut pending, mapping, span)?;
                } else {
                    self.object_work(span, |_, budget| budget.charge(1))?;
                    points
                        .try_reserve(1)
                        .map_err(|_| normalization_limit(span))?;
                    points.push(Point {
                        code: point,
                        class: canonical_combining_class(point),
                    });
                }
            }
        }
        self.order_normalization_points(&mut points, span)?;
        if form.composition() {
            self.compose_normalization_points(&mut points, span)?;
        }
        let mut output = Vec::new();
        for point in points {
            self.tick(span)?;
            if (0xd800..=0xdfff).contains(&point.code) {
                self.append_normalization_units(&mut output, &[point.code as u16], span)?;
            } else {
                let point = char::from_u32(point.code).expect("validated Unicode mappings");
                self.append_normalization_units(
                    &mut output,
                    point.encode_utf16(&mut [0; 2]),
                    span,
                )?;
            }
        }
        Ok(JsString::from_code_units(output))
    }

    fn push_normalization_points(
        &mut self,
        pending: &mut Vec<u32>,
        mapping: &[u32],
        span: Span,
    ) -> Result<(), Error> {
        self.object_work(span, |_, budget| budget.charge(mapping.len()))?;
        pending
            .try_reserve(mapping.len())
            .map_err(|_| normalization_limit(span))?;
        // Explicit expansion stack preserves order without native recursion.
        pending.extend(mapping.iter().rev().copied());
        Ok(())
    }

    fn order_normalization_points(
        &mut self,
        points: &mut [Point],
        span: Span,
    ) -> Result<(), Error> {
        let mut scratch = Vec::new();
        let mut cursor = 0;
        while cursor < points.len() {
            self.tick(span)?;
            if points[cursor].class == 0 {
                cursor += 1;
                continue;
            }
            let start = cursor;
            while cursor < points.len() && points[cursor].class != 0 {
                self.tick(span)?;
                cursor += 1;
            }
            let run = &mut points[start..cursor];
            if run.len() < 2 {
                continue;
            }
            // Stable counting order keeps equal classes in their original order.
            // A fixed 256-class alphabet avoids quadratic combining-mark scans.
            let work = run
                .len()
                .checked_mul(3)
                .and_then(|n| n.checked_add(256))
                .ok_or_else(|| normalization_limit(span))?;
            self.object_work(span, |_, budget| budget.charge(work))?;
            let mut offsets = [0usize; 256];
            for point in run.iter() {
                offsets[usize::from(point.class)] += 1;
            }
            let mut offset = 0;
            for count in &mut offsets {
                let length = *count;
                *count = offset;
                offset += length;
            }
            scratch.clear();
            scratch
                .try_reserve(run.len())
                .map_err(|_| normalization_limit(span))?;
            scratch.resize(run.len(), Point::default());
            for &point in run.iter() {
                let index = &mut offsets[usize::from(point.class)];
                scratch[*index] = point;
                *index += 1;
            }
            run.copy_from_slice(&scratch);
        }
        Ok(())
    }

    fn compose_normalization_points(
        &mut self,
        points: &mut Vec<Point>,
        span: Span,
    ) -> Result<(), Error> {
        let mut starter: Option<usize> = None;
        let mut last_class = 0;
        let mut write = 0;
        for read in 0..points.len() {
            self.tick(span)?;
            let point = points[read];
            if let Some(index) = starter {
                if last_class == 0 || last_class < point.class {
                    if let Some(composite) = canonical_composition(points[index].code, point.code) {
                        points[index].code = composite;
                        // A consumed point does not block the next composition.
                        continue;
                    }
                }
            }
            if point.class == 0 {
                starter = Some(write);
            }
            last_class = point.class;
            points[write] = point;
            write += 1;
        }
        points.truncate(write);
        Ok(())
    }

    fn append_normalization_units(
        &mut self,
        output: &mut Vec<u16>,
        units: &[u16],
        span: Span,
    ) -> Result<(), Error> {
        let length = output
            .len()
            .checked_add(units.len())
            .ok_or_else(|| normalization_limit(span))?;
        if self
            .limits
            .max_string_units
            .is_some_and(|limit| length > limit)
        {
            return Err(normalization_limit(span));
        }
        self.object_work(span, |_, budget| budget.charge(units.len()))?;
        output
            .try_reserve(units.len())
            .map_err(|_| normalization_limit(span))?;
        output.extend_from_slice(units);
        Ok(())
    }
}

fn normalization_limit(span: Span) -> Error {
    Error::Limit {
        span,
        message: "normalization exceeds host string limit or platform capacity".into(),
    }
}
