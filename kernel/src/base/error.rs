pub macro ensure($cond:expr, $err:expr) {
    if !$cond {
        return Err($err);
    }
}

pub trait ErrorOut: Sized {
    type T;
    type U;
    fn try_out<S: TryFrom<Self::U>>(self) -> Result<Result<Self::T, S>, S::Error>;
    fn out<S: TryFrom<Self::U>>(self) -> Result<Self::T, S> {
        match self.try_out() {
            Ok(e) => e,
            Err(_) => panic!(),
        }
    }
}

impl<T, U> ErrorOut for Result<T, U> {
    type T = T;
    type U = U;
    fn try_out<S: TryFrom<U>>(self) -> Result<Result<Self::T, S>, S::Error> {
        match self {
            Ok(x) => Ok(Ok(x)),
            Err(e) => e.try_into().map(Err),
        }
    }
}

pub macro fully {
    ($src: ty, $dst: ty; $($variant: ident),+) => {
        impl From<$src> for $dst {
            fn from(src: $src) -> Self {
                match src {
                    $(<$src>::$variant => Self::$variant,)*
                }
            }
        }
    },
    ($src: ty, $dst: ty; $($src_variant: ident => $dst_variant: ident),+) => {
        impl From<$src> for $dst {
            fn from(src: $src) -> Self {
                match src {
                    $(<$src>::$src_variant => Self::$dst_variant,)*
                }
            }
        }
    }
}

pub macro partially {
    ($src: ty, $dst: ty; $($variant: ident),+) => {
        impl TryFrom<$src> for $dst {
            type Error = ();
            fn try_from(src: $src) -> Result<Self, Self::Error> {
                match src {
                    $(<$src>::$variant => Ok(Self::$variant),)*
                    _ => Err(()),
                }
            }
        }
    },
    ($src: ty, $dst: ty; $($src_variant: ident => $dst_variant: ident),+) => {
        impl TryFrom<$src> for $dst {
            type Error = ();
            fn try_from(src: $src) -> Result<Self, Self::Error> {
                match src {
                    $(<$src>::$src_variant => Ok(Self::$dst_variant),)*
                    _ => Err(()),
                }
            }
        }
    }
}
