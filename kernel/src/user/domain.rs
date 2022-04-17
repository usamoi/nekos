use crate::prelude::*;

impl DomainError for ! {
    fn into_u8(self) -> u8 {
        self
    }
}

impl Domain for () {
    type Error = !;
    fn from_arguments(_: &Environment, _: usize) -> Flow<Self, Either<GeneralError, Self::Error>> {
        Flow::Ok(())
    }
}

impl Domain for usize {
    type Error = !;
    fn from_arguments(_: &Environment, x: usize) -> Flow<Self, Either<GeneralError, Self::Error>> {
        Flow::Ok(x)
    }
}

impl Domain for isize {
    type Error = !;
    fn from_arguments(_: &Environment, x: usize) -> Flow<Self, Either<GeneralError, Self::Error>> {
        Flow::Ok(x as isize)
    }
}

impl Domain for VAddr {
    type Error = !;
    fn from_arguments(_: &Environment, x: usize) -> Flow<Self, Either<GeneralError, Self::Error>> {
        Flow::Ok(VAddr::new(x))
    }
}

#[repr(u8)]
pub enum DomainPermissionError {
    Invaild = 0,
}

impl DomainError for DomainPermissionError {
    fn into_u8(self) -> u8 {
        self as u8
    }
}

impl Domain for Permission {
    type Error = DomainPermissionError;
    fn from_arguments(_: &Environment, x: usize) -> Flow<Self, Either<GeneralError, Self::Error>> {
        if x & !0b111 == 0 {
            Flow::Ok(Self {
                read: (x & 0b001) != 0,
                write: (x & 0b010) != 0,
                execute: (x & 0b100) != 0,
            })
        } else {
            Flow::Err(DomainPermissionError::Invaild.into())
        }
    }
}

#[repr(u8)]
pub enum DomainHandleError {
    NotFound = 0,
}

impl DomainError for DomainHandleError {
    fn into_u8(self) -> u8 {
        self as u8
    }
}

impl Domain for Handle {
    type Error = DomainHandleError;
    fn from_arguments(
        env: &Environment,
        x: usize,
    ) -> Flow<Self, Either<GeneralError, Self::Error>> {
        if let Some(handle) = env.process.handle_set.lookup(x) {
            Flow::Ok(handle)
        } else {
            Flow::Err(DomainHandleError::NotFound.into())
        }
    }
}

#[repr(u8)]
pub enum DomainHandleTError {
    NotFound = 0,
    BadType = 1,
}

impl DomainError for DomainHandleTError {
    fn into_u8(self) -> u8 {
        self as u8
    }
}

impl<T: Object> Domain for Handle<T> {
    type Error = DomainHandleTError;
    fn from_arguments(
        env: &Environment,
        x: usize,
    ) -> Flow<Self, Either<GeneralError, Self::Error>> {
        let handle = env
            .process
            .handle_set
            .lookup(x)
            .ok_or(DomainHandleTError::NotFound)?;
        Flow::Ok(handle.downcast().ok_or(DomainHandleTError::BadType)?)
    }
}
