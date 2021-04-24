use std::convert::TryFrom;

enum Spray {
    Spray1,
    Spray2,
}

impl TryFrom<u32> for Spray {
    type Error = Error;
    fn try_from(num: u32) -> Result<Spray, Self::Error> {
        if num == 0 {
            Ok(Spray::Spray1)
        } else if num == 1 {
            Ok(Spray::Spray2)
        } else {
            Err(Error::UnknownSpray)
        }
    }
}

struct Emit {
    spray: Spray,
    duration: u32,
}

struct Wait {
    duration: u32
}

struct SetFanRPM {
    rpm: u32
}

pub struct State {
    emits: Vec<Emit>,
    waits: Vec<Wait>,
    fans: Vec<SetFanRPM>,
}

impl State {
    pub fn emit_time(&self) -> u32 {
        self.emits
            .iter()
            .fold(0 as u32, |time, emit| time + emit.duration)
    }

    /// returns execution time of experiment in milliseconds
    pub fn execution_time(&self) -> u32 {
        let execution_time = self.emits
            .iter()
            .fold(0 as u32, |time, emit| time + emit.duration);

        self.waits
            .iter()
            .fold(execution_time, |time, wait| time + wait.duration)
    }
}

pub struct Decoder;

impl Decoder {
    pub fn decode(input: &str) -> Result<State, Error> {
        let mut emits = Vec::<Emit>::new();
        let mut waits = Vec::<Wait>::new();
        let mut fans = Vec::<SetFanRPM>::new();

        let mut lines = input.split("\n");

        let first_line = lines.next()
            .ok_or(Error::MalformedString)?;

        if first_line != "" {
            return Err(Error::MalformedString);
        }

        let start_delimiter = lines.next()
            .ok_or(Error::MalformedString)?;

        if start_delimiter != "start_delimiter" {
            return Err(Error::MalformedString);
        }

        loop {
            let line = lines.next()
                .ok_or(Error::MalformedString)?;

            if line == "end_delimiter" {
                break;
            }

            if line == "emit" {
                let spray = Spray::try_from(
                    lines.next()
                        .ok_or(Error::MalformedString)?
                        .parse::<u32>()
                        .map_err(|_| Error::MalformedString)?
                )?;

                let duration = lines.next()
                    .ok_or(Error::MalformedString)?
                    .parse::<u32>()
                    .map_err(|_| Error::MalformedString)?;

                emits.push(Emit {
                    spray,
                    duration,
                })
            } else if line == "wait" {
                let duration = lines.next()
                    .ok_or(Error::MalformedString)?
                    .parse::<u32>()
                    .map_err(|_| Error::MalformedString)?;

                waits.push(Wait {
                    duration
                })
            } else if line == "fan" {
                let rpm = lines.next()
                    .ok_or(Error::MalformedString)?
                    .parse::<u32>()
                    .map_err(|_| Error::MalformedString)?;

                fans.push(SetFanRPM {
                    rpm
                })
            } else {
                return Err(Error::UnknownCommand);
            }
        }

        // consume all remaining new lines, if there are some character at new line, do not accept the input
        for line in lines {
            if line != "" {
                return Err(Error::MalformedString);
            }
        }

        Ok(State {
            emits,
            waits,
            fans,
        })
    }
}

#[derive(Debug)]
pub enum Error {
    MalformedString,
    UnknownSpray,
    UnknownCommand,
}