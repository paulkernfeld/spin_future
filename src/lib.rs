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
pub fn until_true<F: Fn() -> bool>(f: F) -> impl Future<Output = ()> {
    UntilTrue(f)
}

struct UntilTrue<F: Fn() -> bool>(F);

impl<F: Fn() -> bool> Future for UntilTrue<F> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.as_mut().0() {
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
}
