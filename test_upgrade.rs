use hyper::upgrade::Upgraded;
use hyper::{body::Incoming, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::io::{AsyncRead, AsyncWrite};

fn test_up(upgraded: Upgraded) {
    let mut io = TokioIo::new(upgraded);
    let _: &mut dyn AsyncRead = &mut io;
    let _: &mut dyn AsyncWrite = &mut io;
}
