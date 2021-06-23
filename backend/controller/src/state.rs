
pub const START_DELIMITER_NEW_LINE: &str = "start_delimiter\n";
pub const END_DELIMITER_NEW_LINE: &str = "end_delimiter\n";

pub const START_DELIMITER: &str = "start_delimiter";
pub const END_DELIMITER: &str = "end_delimiter";

const NUM_SPRAY: usize = 2;

trait Encode {
    fn encode(&self) -> String;
}

struct Emit {
    sprays: [bool; NUM_SPRAY],
    duration: u32,
}

impl Encode for Emit {
    fn encode(&self) -> String {
        let emits = self.sprays.iter()
            .fold(String::from(""), |res, spray| {
                res + if *spray { "1" } else { "0" }
            });
        format!("emit\n{}\n{}\n", emits, self.duration)
    }
}

struct Wait {
    duration: u32,
}

impl Encode for Wait {
    fn encode(&self) -> String {
        format!("wait\n{}\n", self.duration)
    }
}

struct SetFanRPM {
    rpm: u32,
}

impl Encode for SetFanRPM {
    fn encode(&self) -> String {
        format!("fan\n{}\n", self.rpm)
    }
}

enum Command {
    Emit,
    Wait,
    SetFanRPM,
}

pub struct State {
    emits: Vec<Emit>,
    waits: Vec<Wait>,
    fans: Vec<SetFanRPM>,
    // This indicates the execution order
    order: Vec<Command>,
}

impl State {
    pub fn emit_time(&self) -> u32 {
        self.emits
            .iter()
            .fold(0 as u32, |time, emit| time + emit.duration)
    }

    pub fn execution_time(&self) -> u32 {
        self.waits
            .iter()
            .fold(self.emit_time(), |time, wait| time + wait.duration)
    }
}

impl<'a> IntoIterator for &'a State {
    type Item = String;
    type IntoIter = StateIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        StateIterator {
            emit_iterator: self.emits.iter(),
            wait_iterator: self.waits.iter(),
            fan_iterator: self.fans.iter(),
            order_iterator: self.order.iter(),
        }
    }
}


pub struct StateIterator<'a> {
    emit_iterator: std::slice::Iter<'a, Emit>,
    wait_iterator: std::slice::Iter<'a, Wait>,
    fan_iterator: std::slice::Iter<'a, SetFanRPM>,
    order_iterator: std::slice::Iter<'a, Command>,
}

impl<'a> Iterator for StateIterator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let command = self.order_iterator.next()?;

        let output = match command {
            Command::Emit => self.emit_iterator.next()?.encode(),
            Command::Wait => self.wait_iterator.next()?.encode(),
            Command::SetFanRPM => self.fan_iterator.next()?.encode()
        };

        Some(output)
    }
}

pub struct Decoder;

impl Decoder {
    pub fn decode(input: &str) -> Result<State, Error> {
        let mut emits = Vec::<Emit>::new();
        let mut waits = Vec::<Wait>::new();
        let mut fans = Vec::<SetFanRPM>::new();
        let mut order = Vec::<Command>::new();

        let mut lines = input.split("\n");

        let first_line = lines.next()
            .ok_or(Error::MalformedInput)?;

        if first_line != "" {
            return Err(Error::MalformedInput);
        }

        let start_delimiter = lines.next()
            .ok_or(Error::MalformedInput)?;

        if start_delimiter != START_DELIMITER {
            return Err(Error::MalformedInput);
        }

        loop {
            let line = lines.next()
                .ok_or(Error::MalformedInput)?;

            if line == END_DELIMITER {
                break;
            }

            if line == "emit" {
                let spray_emits: &str = lines.next()
                    .ok_or(Error::MalformedInput)?;

                if spray_emits.len() != NUM_SPRAY {
                    return Err(Error::MalformedInput);
                }

                let mut sprays = [false; NUM_SPRAY];

                spray_emits.char_indices()
                    .for_each(|(i, emit)| sprays[i] = emit == '1');

                let duration = lines.next()
                    .ok_or(Error::MalformedInput)?
                    .parse::<u32>()
                    .map_err(|_| Error::MalformedInput)?;

                emits.push(Emit {
                    sprays,
                    duration,
                });
                order.push(Command::Emit);
            } else if line == "wait" {
                let duration = lines.next()
                    .ok_or(Error::MalformedInput)?
                    .parse::<u32>()
                    .map_err(|_| Error::MalformedInput)?;

                waits.push(Wait {
                    duration
                });
                order.push(Command::Wait);
            } else if line == "fan" {
                let rpm = lines.next()
                    .ok_or(Error::MalformedInput)?
                    .parse::<u32>()
                    .map_err(|_| Error::MalformedInput)?;

                fans.push(SetFanRPM {
                    rpm
                });
                order.push(Command::SetFanRPM);
            } else {
                return Err(Error::UnknownCommand);
            }
        }

        // consume all remaining new lines, if there are some characters at new line, do not accept the input
        for line in lines {
            if line != "" {
                return Err(Error::MalformedInput);
            }
        }

        Ok(State {
            emits,
            waits,
            fans,
            order,
        })
    }
}

#[derive(Debug)]
pub enum Error {
    MalformedInput,
    UnknownCommand,
}