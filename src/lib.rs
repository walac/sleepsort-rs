use async_timer::oneshot::{Oneshot, Timer};
use futures::future::{join_all, ready};
use futures::prelude::*;
use num::cast::AsPrimitive;
use num::Unsigned;
use std::future::Future;
use std::iter::Iterator;
use std::time::Duration;

/// sleep_sort call the passed closure in the ascending sorted order
/// of the items in the iterator using the infamous sleep-sort algorithm
pub fn sleep_sort<T, I, F>(it: I, f: F) -> impl Future<Output = ()>
where
    T: Unsigned + AsPrimitive<u64> + Copy,
    I: Iterator<Item = T>,
    F: Fn(T) + Copy,
{
    join_all(it.map(|n| {
        Timer::new(Duration::from_secs(n.as_())).then(move |_| {
            f(n);
            ready(())
        })
    }))
    .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::future::{AbortHandle, Abortable, Aborted};
    use std::cell::RefCell;
    use tokio;

    #[tokio::test]
    async fn it_works() {
        let data = [3u32, 1, 6, 4, 2, 5];
        let sorted = [1u32, 2, 3, 4, 5, 6];
        let it = RefCell::new(sorted.iter());
        sleep_sort(data.iter().cloned(), |n| {
            let expected = it.borrow_mut().next().unwrap();
            assert_eq!(n, *expected);
        })
        .await;
    }

    #[tokio::test]
    async fn cancellable() {
        let data = [3u32, 1, 6, 4, 2, 5];
        let (abort_handle, abort_registration) = AbortHandle::new_pair();

        let sort_fut = sleep_sort(data.iter().cloned(), |n| {
            println!("{}", n);
        });
        let sort_fut = Abortable::new(sort_fut, abort_registration);

        abort_handle.abort();
        assert_eq!(sort_fut.await, Err(Aborted));
    }
}
