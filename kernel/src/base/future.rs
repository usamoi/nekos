use core::future::Future;
use core::pin::Pin;
use core::task::Poll;

pub fn yield_now() -> impl Future<Output = ()> {
    struct Yield(bool);
    impl Future for Yield {
        type Output = ();
        fn poll(mut self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
            if !self.0 {
                self.0 = true;
                cx.waker().wake_by_ref();
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        }
    }
    Yield(false)
}
