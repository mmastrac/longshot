use crate::prelude::*;

use std::process::Stdio;

use async_stream::stream;
use futures::TryFutureExt;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::ChildStdin,
    sync::Mutex,
};
use tokio_stream::wrappers::LinesStream;

use crate::{
    ecam::{AsyncFuture, EcamDriver, EcamError, EcamOutput, EcamPacketReceiver},
    protocol,
    protocol::*,
};

pub struct EcamSubprocess {
    stdin: Arc<Mutex<ChildStdin>>,
    receiver: EcamPacketReceiver,
}

impl EcamSubprocess {
    async fn write_stdin(&self, data: Vec<u8>) -> Result<(), EcamError> {
        self.stdin
            .lock()
            .await
            .write(format!("S: {}\n", protocol::stringify(&data)).as_bytes())
            .map_ok(|_| ())
            .await?;
        Ok(())
    }
}

impl EcamDriver for EcamSubprocess {
    fn read<'a>(&'a self) -> AsyncFuture<'a, Option<EcamOutput>> {
        Box::pin(self.receiver.recv())
    }

    fn write<'a>(&'a self, data: Vec<u8>) -> AsyncFuture<'a, ()> {
        Box::pin(self.write_stdin(data))
    }

    fn scan<'a>() -> AsyncFuture<'a, (String, uuid::Uuid)>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

pub async fn stream(
    mut child: tokio::process::Child,
) -> Result<impl StreamExt<Item = EcamOutput>, EcamError> {
    let mut stderr =
        LinesStream::new(BufReader::new(child.stderr.take().expect("stderr was missing")).lines());
    let mut stdout =
        LinesStream::new(BufReader::new(child.stdout.take().expect("stdout was missing")).lines());

    let stdout = stream! {
        while let Some(Ok(s)) = stdout.next().await {
            if s == "R: READY" {
                yield EcamOutput::Ready;
            } else if s.starts_with("R: ") {
                if let Ok(bytes) = hex::decode(&s[3..]) {
                    yield EcamOutput::Packet(EcamPacket::from_bytes(&bytes));
                } else {
                    trace_packet!("Failed to decode '{}'", s);
                }
            } else {
                trace_packet!("{}", s);
            }
        }
    };
    let stderr = stream! {
        while let Some(Ok(s)) = stderr.next().await {
            trace_packet!("{}", s);
        }
        // TODO: we might have to spawn this
        if false {
            yield EcamOutput::Ready;
        }
    };

    let termination = stream! {
        let _ = child.wait().await;
        yield EcamOutput::Done
    };

    Result::Ok(stdout.merge(stderr).merge(termination))
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
    let mut child = cmd.spawn()?;
    let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin was missing")));

    let s = Box::pin(stream(child).await?);
    Result::Ok(EcamSubprocess {
        stdin,
        receiver: EcamPacketReceiver::from_stream(s, false),
    })
}
