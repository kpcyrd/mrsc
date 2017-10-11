/*!
  mpsc with requests

  # Examples

  ```
  use mrsc;
  use std::thread;

  let server: mrsc::Server<u32, String> = mrsc::Server::new();
  let channel = server.pop();

  thread::spawn(move || {
      let req = server.recv().unwrap();
      let reply = {
          let msg = req.get();
          assert_eq!(msg, &123);
          "world".to_string()
      };
      req.reply(reply).unwrap();
  });

  let response = channel.req(123).unwrap();
  let reply = response.recv().unwrap();
  assert_eq!(reply, "world".to_string());
*/

use std::sync::mpsc;
use std::time::Duration;

type Sender<T, R> = mpsc::Sender<Request<T, R>>;
type InternalReceiver<T, R> = mpsc::Receiver<Request<T, R>>;
type Receiver<R> = mpsc::Receiver<R>;

pub type SendError<R, T> = mpsc::SendError<Request<R, T>>;
pub type RecvError = mpsc::RecvError;
pub type TryRecvError = mpsc::TryRecvError;
pub type RecvTimeoutError = mpsc::RecvTimeoutError;

/// The server that receives requests and creates channels
#[derive(Debug)]
pub struct Server<T, R> {
    tx: Sender<T, R>,
    rx: InternalReceiver<T, R>,
}

impl<T, R> Server<T, R> {
    /// Create a new server.
    ///
    /// # Examples
    ///
    /// ```
    /// use mrsc;
    ///
    /// let server: mrsc::Server<u32, String> = mrsc::Server::new();
    pub fn new() -> Server<T, R> {
        let (tx, rx) = mpsc::channel();
        Server {
            tx,
            rx,
        }
    }

    /// Request a new channel for a worker thread.
    ///
    /// A channel can safely be cloned without calling pop() for every worker.
    ///
    /// # Examples
    ///
    /// ```
    /// use mrsc;
    ///
    /// let server: mrsc::Server<u32, String> = mrsc::Server::new();
    /// let channel = server.pop();
    pub fn pop(&self) -> Channel<T, R> {
        Channel {
            tx: self.tx.clone(),
        }
    }

    /// Receive a request from a worker thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use mrsc;
    ///
    /// let server: mrsc::Server<u32, String> = mrsc::Server::new();
    ///
    /// let channel = server.pop();
    /// // send request
    /// let response = channel.req(123).unwrap();
    ///
    /// // receive request
    /// let req = server.recv().unwrap();
    pub fn recv(&self) -> Result<Request<T, R>, RecvError> {
        self.rx.recv()
    }

    pub fn try_recv(&self) -> Result<Request<T, R>, TryRecvError> {
        self.rx.try_recv()
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Result<Request<T, R>, RecvTimeoutError> {
        self.rx.recv_timeout(timeout)
    }
}

/// A channel to the server that can be used to send requests
#[derive(Debug, Clone)]
pub struct Channel<T, R> {
    tx: Sender<T, R>,
}

impl<T, R> Channel<T, R> {
    /// Sends a new request to the server.
    ///
    /// # Examples
    ///
    /// ```
    /// use mrsc;
    ///
    /// let server: mrsc::Server<u32, String> = mrsc::Server::new();
    ///
    /// let channel = server.pop();
    /// channel.req(123).unwrap();
    pub fn req(&self, payload: T) -> Result<Response<R>, SendError<T, R>> {
        let (tx, rx) = mpsc::channel();
        self.tx.send(Request {
            tx,
            payload
        })?;

        Ok(Response {
            rx: rx
        })
    }
}

/// The request as seen by the server thread
#[derive(Debug)]
pub struct Request<T, R> {
    tx: mpsc::Sender<R>,
    payload: T,
}

impl<T, R> Request<T, R> {
    /// Returns a reference to the request payload.
    pub fn get(&self) -> &T {
        &self.payload
    }

    /// Returns the payload and an EmptyRequest, consumes the Request.
    pub fn take(self) -> (EmptyRequest<R>, T) {
        (EmptyRequest {
            tx: self.tx,
        }, self.payload)
    }

    /// Reply to the request with a response. This consumes the request.
    ///
    /// # Examples
    ///
    /// ```
    /// use mrsc;
    ///
    /// let server: mrsc::Server<u32, String> = mrsc::Server::new();
    ///
    /// let channel = server.pop();
    /// // send request
    /// let response = channel.req(123).unwrap();
    ///
    /// // answer request
    /// let req = server.recv().unwrap();
    /// req.reply("hello world".to_string()).unwrap();
    pub fn reply(self, response: R) -> Result<(), mpsc::SendError<R>> {
        self.tx.send(response)
    }
}

/// A request without payload
#[derive(Debug)]
pub struct EmptyRequest<R> {
    tx: mpsc::Sender<R>,
}

impl<R> EmptyRequest<R> {
    /// Reply to the request with a response. This consumes the request.
    ///
    /// # Examples
    ///
    /// ```
    /// use mrsc;
    ///
    /// let server: mrsc::Server<u32, String> = mrsc::Server::new();
    ///
    /// let channel = server.pop();
    /// // send request
    /// let response = channel.req(123).unwrap();
    ///
    /// // answer request
    /// let req = server.recv().unwrap();
    /// let (req, payload) = req.take();
    /// req.reply("hello world".to_string()).unwrap();
    pub fn reply(self, response: R) -> Result<(), mpsc::SendError<R>> {
        self.tx.send(response)
    }
}

/// The response returned to an request
#[derive(Debug)]
pub struct Response<R> {
    rx: Receiver<R>,
}

impl<R> Response<R> {
    /// Receives the response from the server. Blocks until the request has been answered.
    /// Since there is only one response to each request this consumes the Response.
    /// This operation is blocking.
    ///
    /// # Examples
    ///
    /// ```
    /// use mrsc;
    ///
    /// let server: mrsc::Server<u32, String> = mrsc::Server::new();
    ///
    /// let channel = server.pop();
    /// // send request
    /// let response = channel.req(123).unwrap();
    ///
    /// // answer request
    /// server.recv().unwrap().reply("hello world".to_string()).unwrap();
    ///
    /// // receive result
    /// response.recv().unwrap();
    pub fn recv(self) -> Result<R, RecvError> {
        self.rx.recv()
    }

    pub fn try_recv(&self) -> Result<R, TryRecvError> {
        self.rx.try_recv()
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Result<R, RecvTimeoutError> {
        self.rx.recv_timeout(timeout)
    }
}


#[cfg(test)]
mod tests {
    use super::Server;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn readme() {
        let server: Server<u32, String> = Server::new();
        let channel = server.pop();

        thread::spawn(move || {
            let req = server.recv().unwrap();
            let reply = {
                let msg = req.get();
                println!("request: {:?}", msg);

                "hello world".to_string()
            };
            req.reply(reply).unwrap();
        });

        let response = channel.req(123).unwrap();
        let reply = response.recv().unwrap();
        println!("response: {:?}", reply);
    }

    #[test]
    fn single_requester() {
        let server: Server<u32, String> = Server::new();
        let channel = server.pop();

        thread::spawn(move || {
            for i in &[1, 2, 3] {
                let req = server.recv().unwrap();
                assert_eq!(req.get(), i);
                req.reply(format!("success: {}", i)).unwrap();
            }
        });

        let response = channel.req(1).unwrap();
        let reply = response.recv().unwrap();
        assert_eq!(reply, "success: 1".to_string());

        let response = channel.req(2).unwrap();
        let reply = response.recv().unwrap();
        assert_eq!(reply, "success: 2".to_string());

        let response = channel.req(3).unwrap();
        let reply = response.recv().unwrap();
        assert_eq!(reply, "success: 3".to_string());
    }

    #[test]
    fn take() {
        let server: Server<u32, String> = Server::new();
        let channel = server.pop();

        thread::spawn(move || {
            for i in &[1] {
                let req = server.recv().unwrap();
                let (req, payload) = req.take();
                assert_eq!(&payload, i);
                req.reply(format!("success: {}", i)).unwrap();
            }
        });

        let response = channel.req(1).unwrap();
        let reply = response.recv().unwrap();
        assert_eq!(reply, "success: 1".to_string());
    }

    #[test]
    fn try_recv() {
        let server: Server<u32, u32> = Server::new();
        let channel = server.pop();

        assert!(server.try_recv().is_err());
        let response = channel.req(1).unwrap();
        assert!(response.try_recv().is_err());

        let req = server.try_recv().unwrap();
        let (req, value) = req.take();
        req.reply(value + 2).unwrap();

        let result = response.recv().unwrap();
        assert_eq!(result, 3);
    }

    #[test]
    fn recv_timeout() {
        let server: Server<u32, u32> = Server::new();
        let channel = server.pop();

        let duration = Duration::from_secs(1);

        assert!(server.recv_timeout(duration.clone()).is_err());
        let response = channel.req(1).unwrap();
        assert!(response.recv_timeout(duration.clone()).is_err());

        let req = server.recv_timeout(duration.clone()).unwrap();
        let (req, value) = req.take();
        req.reply(value + 2).unwrap();

        let result = response.recv().unwrap();
        assert_eq!(result, 3);
    }
}
