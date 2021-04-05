from enum import IntEnum

start_delimiter = 'start_delimiter'
end_delimiter = 'end_delimiter'


class Spray(IntEnum):
    Spray_1 = 0
    Spray_2 = 1


class Encoders(IntEnum):
    ByteEncoder = 0


class Encoder:
    id: int

    def encode(self, state) -> str:
        """Encode given state into string"""
        pass


class Command:
    pass


class Emit(Command):
    def __init__(self, spray: Spray, duration: int):
        self.spray = spray
        self.duration = duration


class Wait(Command):
    def __init__(self, duration: int):
        self.duration = duration


class SetFanRPM(Command):
    def __init__(self, rpm: int):
        self.rpm = rpm


class State:
    def __init__(self, encoder: Encoder):
        self.commands = []
        self.encoder = encoder

    def emit(self, spray: Spray, duration: int):
        self.commands.append(Emit(spray, duration))

    def wait(self, duration: int):
        self.commands.append(Wait(duration))

    def set_fan_rpm(self, rpm: int):
        self.commands.append(SetFanRPM(rpm))

    def execute(self):
        print(self.encoder.encode(self))


class WordEncoder(Encoder):
    id: int = int(Encoders.ByteEncoder)

    def encode(self, state: State) -> str:
        output = start_delimiter + '\n' + str(WordEncoder.id) + '\n'
        for act in state.commands:
            output += self._encode_action(act) + '\n'

        return output + 'end_delimiter'

    def _encode_action(self, cmd: Command) -> str:
        if isinstance(cmd, Emit):
            return 'emit\n{}\n{}'.format(int(cmd.spray), cmd.duration)
        elif isinstance(cmd, Wait):
            return 'wait\n{}'.format(int(cmd.duration))
        elif isinstance(cmd, SetFanRPM):
            return 'fan\n{}'.format(int(cmd.rpm))
        else:
            print('unimplemented action {} for {} encoder'.format(cmd.__class__, self.__class__))
            return ''


def run():
    spray_duration = 100  # ms
    pause_duration = 200  # ms

    state = State(WordEncoder())
    state.set_fan_rpm(100)
    state.emit(Spray.Spray_1, spray_duration)
    state.wait(pause_duration)
    state.execute()


if __name__ == '__main__':
    run()
