#[macro_export]
macro_rules! int_rw_register {
    ($reg:ident : $addr:expr, $type:ty, $res:expr) => {
        #[derive(Clone, Debug, PartialEq)]
        pub struct $reg {
            value: Option<$type>,
            resolution: Resolution
        }
        impl $reg {
            fn as_bytes(&self) -> Result<Vec<u8>, RegisterError> {
                let Some(value) = self.value else {
                    return Err(RegisterError::NoData);
                };
                match self.resolution {
                    Resolution::Int8 => value.try_into_1_byte(None).map(|x| vec![x]),
                    Resolution::Int16 => value.try_into_2_bytes(None).map(|x| x.to_vec()),
                    Resolution::Int32 => value.try_into_4_bytes(None).map(|x| x.to_vec()),
                    Resolution::Float => value.try_into_f32_bytes(None).map(|x| x.to_vec()),
                }
            }
        }
        impl From<$reg> for RegisterDataStruct {
            fn from(reg: $reg) -> RegisterDataStruct {
                if let Ok(data) = reg.as_bytes() {
                    return RegisterDataStruct {
                        address: $reg::address(),
                        resolution: reg.resolution,
                        data: Some(data),
                    };
                } else {
                    return RegisterDataStruct {
                        address: $reg::address(),
                        resolution: reg.resolution,
                        data: None,
                    };
                }
            }
        }
        impl Register for $reg {
            fn address() -> RegisterAddr {
                $addr
            }

            fn from_bytes(bytes: &[u8], resolution: Resolution) -> Result<Self, RegisterError> where Self: Sized
            {
                Ok(match resolution {
                    Resolution::Int8 => $reg {
                        value: Some(<$type>::try_from_1_byte(bytes[0], None)?),
                        resolution,
                    },
                    Resolution::Int16 => $reg {
                        value: Some(<$type>::try_from_2_bytes(&bytes[..2], None)?),
                        resolution,
                    },
                    Resolution::Int32 => $reg {
                        value: Some(<$type>::try_from_4_bytes(&bytes[..4], None)?),
                        resolution,
                    },
                    Resolution::Float => $reg {
                        value: Some(<$type>::try_from_f32_bytes(&bytes[..4], None)?),
                        resolution,
                    },
                })
            }
        }
        impl RegisterData<$type> for $reg {
            const DEFAULT_RESOLUTION: Resolution = $res;

            fn write(data: $type) -> Self {
                $reg {
                    value: Some(data),
                    resolution: Self::DEFAULT_RESOLUTION,
                }
            }
            fn write_with_resolution(data: $type, r: Resolution) -> Self {
                $reg {
                    value: Some(data),
                    resolution: r,
                }
            }
            fn read() -> Self {
                $reg {
                    value: None,
                    resolution: Self::DEFAULT_RESOLUTION,
                }
            }
            fn read_with_resolution(r: Resolution) -> Self {
                $reg {
                    value: None,
                    resolution: r,
                }
            }

        }
    };
}

#[macro_export]
macro_rules! map_rw_register {
    ($reg:ident : $addr:expr, $mapping:expr) => {
        #[derive(Clone, Debug, PartialEq)]
        pub struct $reg {
            value: Option<f32>,
            resolution: Resolution,
        }
        impl $reg {
            fn as_bytes(&self) -> Result<Vec<u8>, RegisterError> {
                let Some(value) = self.value else {
                    return Err(RegisterError::NoData);
                };
                match self.resolution {
                    Resolution::Int8 => value.try_into_1_byte(Some($mapping)).map(|x| vec![x]),
                    Resolution::Int16 => value.try_into_2_bytes(Some($mapping)).map(|x| x.to_vec()),
                    Resolution::Int32 => value.try_into_4_bytes(Some($mapping)).map(|x| x.to_vec()),
                    Resolution::Float => value.try_into_f32_bytes(Some($mapping)).map(|x| x.to_vec()),
                }
            }
        }
        impl From<$reg> for RegisterDataStruct {
            fn from(reg: $reg) -> RegisterDataStruct {
                if let Ok(data) = reg.as_bytes() {
                    return RegisterDataStruct {
                        address: $reg::address(),
                        resolution: reg.resolution,
                        data: Some(data),
                    };
                } else {
                    return RegisterDataStruct {
                        address: $reg::address(),
                        resolution: reg.resolution,
                        data: None,
                    };
                }
            }
        }
        impl Register for $reg {
            fn address() -> RegisterAddr {
                $addr
            }

            fn from_bytes(bytes: &[u8], resolution: Resolution) -> Result<Self, RegisterError> where Self: Sized
            {
                Ok(match resolution {
                    Resolution::Int8 => Self {
                        value: Some(f32::try_from_1_byte(bytes[0], Some($mapping))?),
                        resolution,
                    },
                    Resolution::Int16 => Self {
                        value: Some(f32::try_from_2_bytes(&bytes[..2], Some($mapping))?),
                        resolution,
                    },
                    Resolution::Int32 => Self {
                        value: Some(f32::try_from_4_bytes(&bytes[..4], Some($mapping))?),
                        resolution,
                    },
                    Resolution::Float => Self {
                        value: Some(f32::try_from_f32_bytes(&bytes[..4], Some($mapping))?),
                        resolution,
                    },
                })
            }
        }
        impl RegisterData<f32> for $reg {
            const DEFAULT_RESOLUTION: Resolution = Resolution::Float;

            fn write(data: f32) -> Self {
                $reg {
                    value: Some(data),
                    resolution: Self::DEFAULT_RESOLUTION,
                }
            }
            fn write_with_resolution(data: f32, r: Resolution) -> Self {
                $reg {
                    value: Some(data),
                    resolution: r,
                }
            }
            fn read() -> Self {
                $reg {
                    value: None,
                    resolution: Self::DEFAULT_RESOLUTION,
                }
            }
            fn read_with_resolution(r: Resolution) -> Self {
                $reg {
                    value: None,
                    resolution: r,
                }
            }

        }
    };
}
