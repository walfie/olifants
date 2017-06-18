use error::*;
use futures::{Async, Stream};

pub struct Lines<S> {
    stream: S,
    buffer: String,
}

impl<S, B> Lines<S>
where
    B: AsRef<[u8]>,
    S: Stream<Item = B, Error = Error>,
{
    pub fn new(stream: S) -> Self {
        Lines {
            stream,
            buffer: String::new(),
        }
    }
}

// TODO: Add warning if unused
impl<S, B> Stream for Lines<S>
where
    B: AsRef<[u8]>,
    S: Stream<Item = B, Error = Error>,
{
    type Item = String;
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>> {
        loop {
            if let Some(index) = self.buffer.find('\n') {
                let mut split = self.buffer.split_off(index + 1);
                ::std::mem::swap(&mut self.buffer, &mut split);
                return Ok(Async::Ready(Some(split)));
            } else {
                // Attempt to fill the buffer
                if let Some(chunk) = try_ready!(self.stream.poll()) {
                    match ::std::str::from_utf8(&chunk.as_ref()) {
                        Ok(s) => {
                            self.buffer.push_str(s);
                        }
                        Err(e) => {
                            return Err(Error::with_chain(e, ErrorKind::Utf8));
                        }
                    }
                } else {
                    return Ok(Async::Ready(None));
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::{self, Future, Stream};
    use futures::unsync::mpsc;

    #[test]
    #[allow(unused_must_use)]
    fn lines() {
        // Run on a task context
        futures::lazy(|| {
            let (msg_tx, msg_rx) = mpsc::unbounded::<&str>();
            let mut lines = Lines::new(msg_rx.map_err(|_| Error::from_kind(ErrorKind::Http)));

            let send = move |msg| mpsc::UnboundedSender::send(&msg_tx, msg);

            send("First");
            assert_eq!(lines.poll().unwrap(), Async::NotReady);
            send(" line");
            assert_eq!(lines.poll().unwrap(), Async::NotReady);
            send("\nSecond line\nThird line");
            assert_eq!(
                lines.poll().unwrap(),
                Async::Ready(Some("First line\n".to_string()))
            );
            assert_eq!(
                lines.poll().unwrap(),
                Async::Ready(Some("Second line\n".to_string()))
            );
            assert_eq!(lines.poll().unwrap(), Async::NotReady);
            send("\n");
            assert_eq!(
                lines.poll().unwrap(),
                Async::Ready(Some("Third line\n".to_string()))
            );

            Ok::<(), ()>(())
        }).wait()
            .unwrap();
    }
}
