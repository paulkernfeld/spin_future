//! This crate needs rust 1.46 or newer to get around an interesting issue: https://github.com/rust-lang/rust/issues/65489
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

/// Convert a function that returns bool into a valid but very inefficient future.
/// This will return `Poll::Ready` if and only if the function returns true.
/// The key trick
/// to make this valid is that we always call the waker if we are going to return `Pending`.
/// That way the executor is guaranteed to continue polling us. This doesn't actually matter if
/// we're using the `block_on` executor from this mod, but it would matter if we used a normal
/// executor. I got this trick from user HadrienG in [this Rust forum post](https://users.rust-lang.org/t/polling-in-new-era-futures/30531/2).
pub fn until_true<F: FnMut() -> bool + Unpin>(f: F) -> impl Future<Output = ()> {
    UntilTrue(f)
}

struct UntilTrue<F: FnMut() -> bool>(F);

// TODO why do we need to implement Unpin here?
impl<F: FnMut() -> bool + Unpin> Future for UntilTrue<F> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // This is a workaround for earlier versions of Rust
        // if (&mut *self).0() {
        if self.0() {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::until_true;
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
}
