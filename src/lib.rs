//! This crate needs rust 1.46 or newer to get around an interesting issue: https://github.com/rust-lang/rust/issues/65489
//!
//! The key trick to make this valid is that we always call the waker if we are going to return
//! `Pending`. That way the executor is guaranteed to continue polling us. I got this trick from
//! user HadrienG in [this Rust forum post](https://users.rust-lang.org/t/polling-in-new-era-futures/30531/2).
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

/// Convert a function that returns bool into a valid but inefficient future. The future will
/// resolve only when the function returns true.
pub fn until_true<F: FnMut() -> bool + Unpin>(f: F) -> impl Future<Output = ()> {
    UntilTrue(f)
}

struct UntilTrue<F: FnMut() -> bool>(F);

// TODO why do we need to implement Unpin here?
impl<F: FnMut() -> bool + Unpin> Future for UntilTrue<F> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // This is a workaround for versions of Rust < 1.46
        // if (&mut *self).0() {
        if self.0() {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

/// Convert a function that returns `Option<T>` into a valid but inefficient future. The future will
/// resolve only when the function returns `Some`.
pub fn until_some<T, F: FnMut() -> Option<T> + Unpin>(f: F) -> impl Future<Output = T> {
    UntilSome(f)
}

struct UntilSome<T, F: FnMut() -> Option<T>>(F);

// TODO why do we need to implement Unpin here?
impl<T, F: FnMut() -> Option<T> + Unpin> Future for UntilSome<T, F> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(output) = self.0() {
            Poll::Ready(output)
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{until_true, until_some};
    use futures_lite::future::block_on;

    #[test]
    fn until_true_once() {
        block_on(until_true(|| true))
    }

    #[test]
    fn until_true_twice() {
        let mut first_time = true;
        block_on(until_true(|| {
            if first_time {
                first_time = false;
                false
            } else {
                true
            }
        }))
    }

    #[test]
    fn until_some_once() {
        assert_eq!((), block_on(until_some(|| Some(()))));
    }

    #[test]
    fn until_some_twice() {
        let mut first_time = true;
        let resolved = block_on(until_some(|| {
            if first_time {
                first_time = false;
                None
            } else {
                Some(())
            }
        }));
        assert_eq!((), resolved);
    }
}
