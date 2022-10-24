use std::process::Stdio;

use async_stream::stream;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_stream::{wrappers::LinesStream, Stream, StreamExt};

use crate::ecam_bt::EcamError;

pub struct EcamSubprocess {
    child: Option<tokio::process::Child>,
}

#[derive(Debug)]
pub enum EcamSubprocessOutput {
    Ready,
    Packet(Vec<u8>),
    Logging(String),
    Done,
}

impl EcamSubprocess {
    pub async fn read(
        self: &mut Self,
    ) -> Result<impl Stream<Item = EcamSubprocessOutput>, EcamError> {
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
                    yield EcamSubprocessOutput::Ready;
                } else if s.starts_with("R: ") {
                    if let Ok(bytes) = hex::decode(&s[3..]) {
                        yield EcamSubprocessOutput::Packet(bytes);
                    } else {
                        yield EcamSubprocessOutput::Logging(format!("Failed to decode '{}'", s));
                    }
                } else {
                    yield EcamSubprocessOutput::Logging(s);
                }
            }
        };
        let stderr = stream! {
            while let Some(Ok(s)) = stderr.next().await {
                yield EcamSubprocessOutput::Logging(s);
            }
        };

        let termination = stream! {
            let _ = child.wait().await;
            yield EcamSubprocessOutput::Done
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
