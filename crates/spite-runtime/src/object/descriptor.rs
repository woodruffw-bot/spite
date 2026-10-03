//! Complete properties and partial descriptors for ordinary internal methods.

use crate::Value;
use spite_heap::Handle;

/// A complete ordinary data property.
#[derive(Clone, Debug, PartialEq)]
pub struct DataProperty {
    /// The stored value.
    pub value: Value,
    /// Whether assignment may change the value.
    pub writable: bool,
    /// Whether the property participates in enumerable-key operations.
    pub enumerable: bool,
    /// Whether the property may be deleted or reconfigured.
    pub configurable: bool,
}

/// A complete ordinary accessor property.
#[derive(Clone, Debug, PartialEq)]
pub struct AccessorProperty {
    /// The getter function, or undefined.
    pub get: Option<Handle>,
    /// The setter function, or undefined.
    pub set: Option<Handle>,
    /// Whether the property participates in enumerable-key operations.
    pub enumerable: bool,
    /// Whether the property may be deleted or reconfigured.
    pub configurable: bool,
}

/// A fully populated ordinary property, never both data and accessor.
#[derive(Clone, Debug, PartialEq)]
pub enum Property {
    /// A stored value and writable attribute.
    Data(DataProperty),
    /// Getter and setter function identities.
    Accessor(AccessorProperty),
}

impl Property {
    /// Borrows data attributes, or returns None for an accessor.
    pub fn as_data(&self) -> Option<&DataProperty> {
        match self {
            Self::Data(data) => Some(data),
            Self::Accessor(_) => None,
        }
    }

    /// Returns the enumerable attribute regardless of property kind.
    pub fn enumerable(&self) -> bool {
        match self {
            Self::Data(data) => data.enumerable,
            Self::Accessor(accessor) => accessor.enumerable,
        }
    }

    /// Returns the configurable attribute regardless of property kind.
    pub fn configurable(&self) -> bool {
        match self {
            Self::Data(data) => data.configurable,
            Self::Accessor(accessor) => accessor.configurable,
        }
    }

    pub(super) fn apply(&mut self, descriptor: PropertyDescriptor) -> bool {
        // ValidateAndApplyPropertyDescriptor, 10.1.6.3. Validate every field
        // before mutation, including kind changes and frozen accessor identity.
        if !self.configurable() {
            if descriptor.configurable == Some(true)
                || descriptor
                    .enumerable
                    .is_some_and(|value| value != self.enumerable())
            {
                return false;
            }
            match (&*self, &descriptor.kind) {
                (_, DescriptorKind::Generic) => {}
                (Self::Accessor(current), DescriptorKind::Accessor { get, set }) => {
                    if get.as_ref().is_some_and(|value| *value != current.get)
                        || set.as_ref().is_some_and(|value| *value != current.set)
                    {
                        return false;
                    }
                }
                (Self::Data(current), DescriptorKind::Data { value, writable }) => {
                    if !current.writable {
                        if *writable == Some(true) {
                            return false;
                        }
                        if let Some(value) = value {
                            // Preserve existing distinguishable NaN payload bits.
                            return value.same_value(&current.value);
                        }
                    }
                }
                _ => return false,
            }
        }
        let enumerable = descriptor.enumerable.unwrap_or(self.enumerable());
        let configurable = descriptor.configurable.unwrap_or(self.configurable());
        match descriptor.kind {
            DescriptorKind::Generic => match self {
                Self::Data(data) => {
                    data.enumerable = enumerable;
                    data.configurable = configurable;
                }
                Self::Accessor(accessor) => {
                    accessor.enumerable = enumerable;
                    accessor.configurable = configurable;
                }
            },
            DescriptorKind::Data { value, writable } => {
                let (previous, was_writable) = match self {
                    Self::Data(data) => (
                        std::mem::replace(&mut data.value, Value::Undefined),
                        data.writable,
                    ),
                    Self::Accessor(_) => (Value::Undefined, false),
                };
                *self = Self::Data(DataProperty {
                    value: value.unwrap_or(previous),
                    writable: writable.unwrap_or(was_writable),
                    enumerable,
                    configurable,
                });
            }
            DescriptorKind::Accessor { get, set } => {
                let (previous_get, previous_set) = match self {
                    Self::Accessor(accessor) => (accessor.get.take(), accessor.set.take()),
                    Self::Data(_) => (None, None),
                };
                *self = Self::Accessor(AccessorProperty {
                    get: get.unwrap_or(previous_get),
                    set: set.unwrap_or(previous_set),
                    enumerable,
                    configurable,
                });
            }
        }
        true
    }
}

/// The kind-specific fields of a partial descriptor.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum DescriptorKind {
    /// No value, writable, get, or set field is present.
    #[default]
    Generic,
    /// Data fields. An omitted field preserves its previous value when applicable.
    Data {
        /// A replacement value.
        value: Option<Value>,
        /// A replacement writable attribute.
        writable: Option<bool>,
    },
    /// Accessor fields. Outer None omits a field; Some(None) supplies undefined.
    Accessor {
        /// A replacement getter identity, or undefined.
        get: Option<Option<Handle>>,
        /// A replacement setter identity, or undefined.
        set: Option<Option<Handle>>,
    },
}

/// A partial ordinary descriptor with mutually exclusive data and accessor fields.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PropertyDescriptor {
    /// Kind-specific fields; empty data/accessor variants are normalized to generic.
    pub kind: DescriptorKind,
    /// A replacement enumerable attribute, if supplied.
    pub enumerable: Option<bool>,
    /// A replacement configurable attribute, if supplied.
    pub configurable: Option<bool>,
}

impl PropertyDescriptor {
    pub(super) fn normalize(mut self) -> Self {
        if matches!(
            self.kind,
            DescriptorKind::Data {
                value: None,
                writable: None
            } | DescriptorKind::Accessor {
                get: None,
                set: None
            }
        ) {
            self.kind = DescriptorKind::Generic;
        }
        self
    }

    pub(super) fn complete(self) -> Property {
        let mut property = Property::Data(DataProperty {
            value: Value::Undefined,
            writable: false,
            enumerable: false,
            configurable: true,
        });
        // Use a configurable temporary so either property kind can be installed.
        property.apply(Self {
            configurable: Some(self.configurable.unwrap_or(false)),
            ..self
        });
        property
    }
}

/// A partial data or generic descriptor; omitted fields remain unspecified.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DataDescriptor {
    /// A replacement value, if supplied.
    pub value: Option<Value>,
    /// A replacement writable attribute, if supplied.
    pub writable: Option<bool>,
    /// A replacement enumerable attribute, if supplied.
    pub enumerable: Option<bool>,
    /// A replacement configurable attribute, if supplied.
    pub configurable: Option<bool>,
}

impl From<DataDescriptor> for PropertyDescriptor {
    fn from(data: DataDescriptor) -> Self {
        Self {
            kind: DescriptorKind::Data {
                value: data.value,
                writable: data.writable,
            },
            enumerable: data.enumerable,
            configurable: data.configurable,
        }
        .normalize()
    }
}
