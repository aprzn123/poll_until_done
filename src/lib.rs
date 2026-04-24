use std::{pin::Pin, sync::{Arc, Mutex, mpsc}, task::{Context, Poll, Wake, Waker}};

struct Task<T> {
    fut: Mutex<Pin<Box<dyn Future<Output = T> + Send + Sync>>>,
    awakener: mpsc::SyncSender<()>,
}

impl<T> Wake for Task<T> {
    fn wake(self: Arc<Self>) {
        println!("{:?}", self.awakener.send(()));
    }
}

pub trait SyncAwait: Future {
    fn run(self) -> Self::Output;
}

impl<T> SyncAwait for T where T: Future + 'static + Send + Sync {
    fn run(self) -> Self::Output {
        let (tx, rx) = mpsc::sync_channel::<()>(1);
        let task = Arc::new(Task { fut: Mutex::new(Box::pin(self)), awakener: tx });
        let waker = Waker::from(task.clone());
        let mut ctx = Context::from_waker(&waker);
        loop {
            if let Poll::Ready(v) = task.fut.lock().unwrap().as_mut().poll(&mut ctx) {
                break v;
            }
            println!("{:?}", rx.recv());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_a_future() {
        let n = async {
            async {5}.await + 5
        }.run();
        assert_eq!(n, 10);
    }
}
