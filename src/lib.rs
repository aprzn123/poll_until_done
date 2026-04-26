use std::{mem, pin::pin, sync::mpsc::{self, SyncSender}, task::{Context, Poll, RawWaker, RawWakerVTable, Waker}};

// Our custom waker doesn't semantically own its SyncSender, it just holds a reference to it.
// The Waker is dropped before the Sender, so everything is so easy actually.

unsafe fn waker_clone(p: *const ()) -> RawWaker {
    RawWaker::new(p, &WAKER_VTABLE)
}

unsafe fn waker_wake(p: *const ()) {
    let tx = unsafe {&*(p as *const SyncSender<()>)};
    let _ = tx.send(());
}

unsafe fn waker_wake_by_ref(p: *const ()) {
    let tx = unsafe {&*(p as *const SyncSender<()>)};
    let _ = tx.send(());
}

unsafe fn waker_drop(p: *const ()) {
}

static WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(waker_clone, waker_wake, waker_wake_by_ref, waker_drop);

pub trait SyncAwait: Future {
    fn run(self) -> Self::Output;
}

impl<T> SyncAwait for T where T: Future + 'static + Send + Sync {
    fn run(self) -> Self::Output {
        let (tx, rx) = mpsc::sync_channel::<()>(1);
        let waker = unsafe {Waker::new(&tx as *const SyncSender<()> as *const (), &WAKER_VTABLE)};

        let mut ctx = Context::from_waker(&waker);

        let mut fut = pin!(self);
        let out = loop {
            if let Poll::Ready(v) = fut.as_mut().poll(&mut ctx) {
                break v;
            }
            let _ = rx.recv();
        };
        // ensure that tx outlives waker (kept alive by `ctx`, used only until end of loop)
        // not sure if this is necessary for safety but it certainly doesn't hurt
        mem::drop(tx);
        out
    }
}

#[cfg(test)]
mod tests {
    use futures::join;
    use futures_time::{task::sleep, time::Duration};

    use super::*;

    #[test]
    fn run_a_future() {
        let n = async {
            async {5}.await + 5
        }.run();
        assert_eq!(n, 10);
    }

    #[test]
    fn run_two_futures() {
        let (m, n, _, _) = async { join!(
            async { async { 5 }.await + 5 }, 
            async { 1 + async { 1 }.await },
            sleep(Duration::from_secs(1)),
            sleep(Duration::from_secs(2)),
            ) }.run();
        assert_eq!(m, 10);
        assert_eq!(n, 2);
    }
}
