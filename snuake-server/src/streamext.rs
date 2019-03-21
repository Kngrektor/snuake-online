use futures::stream::Stream;
use futures::{Poll, Async};


#[must_use = "streams do nothing unless polled"]
pub struct EndOnError<S> {
    stream: S,
    done: bool,
}

impl<S: Stream> Stream for EndOnError<S> {
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<S::Item>, S::Error> {
        if self.done {
            Ok(Async::Ready(None))
        } else {
            match self.stream.poll() {
                Err(_) => {
                    self.done = true;
                    Ok(Async::Ready(None))
                }
                r => r
            }
        }
    }
}


pub trait StreamExt: Stream + Sized {
    fn end_on_error(self) -> EndOnError<Self> {
        EndOnError { stream: self, done: false }
    }
}

impl<T> StreamExt for T where T: Stream {}