use std::process::Stdio;

use async_stream::stream;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_stream::{wrappers::LinesStream, Stream, StreamExt};

use crate::{
    command::Response,
    ecam::{EcamError, EcamOutput},
};

pub struct EcamSubprocess {
    child: Option<tokio::process::Child>,
}

impl EcamSubprocess {
    pub async fn read(self: &mut Self) -> Result<impl Stream<Item = EcamOutput>, EcamError> {
        let mut child = self.child.take().expect("child was missing");
        let mut stderr = LinesStream::new(
            BufReader::new(child.stderr.take().expect("stderr was missing")).lines(),
        );
        let mut stdout = LinesStream::new(
            BufReader::new(child.stdout.take().expect("stdout was missing")).lines(),
        );

        let stdout = stream! {
            while let Some(Ok(s)) = stdout.next().await {
                if s == "R: READY" {
                    yield EcamOutput::Ready;
                } else if s.starts_with("R: ") {
                    if let Ok(bytes) = hex::decode(&s[3..]) {
                        yield EcamOutput::Packet(Response::decode(&bytes));
                    } else {
                        yield EcamOutput::Logging(format!("Failed to decode '{}'", s));
                    }
                } else {
                    yield EcamOutput::Logging(s);
                }
            }
        };
        let stderr = stream! {
            while let Some(Ok(s)) = stderr.next().await {
                yield EcamOutput::Logging(s);
            }
        };

        let termination = stream! {
            let _ = child.wait().await;
            yield EcamOutput::Done
        };

        Result::Ok(stdout.merge(stderr).merge(termination))
    }
}

pub async fn connect(device_name: &str) -> Result<EcamSubprocess, EcamError> {
    let mut cmd = tokio::process::Command::new(std::env::current_exe()?);
    cmd.arg("x-internal-pipe");
    cmd.arg("--device-name");
    cmd.arg(device_name);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.kill_on_drop(true);
    let child = cmd.spawn()?;

    Result::Ok(EcamSubprocess { child: Some(child) })
}
