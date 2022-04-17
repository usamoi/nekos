#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Either<T, U> {
    Left(T),
    Right(U),
}

pub use Either::{Left, Right};

impl<T, U> Either<T, U> {
    pub const fn is_left(&self) -> bool {
        match self {
            Left(_) => true,
            Right(_) => false,
        }
    }
    pub const fn is_right(&self) -> bool {
        match self {
            Left(_) => false,
            Right(_) => true,
        }
    }
    pub fn unwrap_left(self) -> T {
        match self {
            Left(x) => x,
            Right(_) => panic!("called `Either::unwrap_left()` on an `Right` value"),
        }
    }
    pub fn unwrap_right(self) -> U {
        match self {
            Left(_) => panic!("called `Either::unwrap_right()` on an `Left` value"),
            Right(x) => x,
        }
    }
}

impl<T, U> From<U> for Either<T, U> {
    fn from(x: U) -> Self {
        Right(x)
    }
}
