from enum import IntEnum


class Spray(IntEnum):
    SPRAY_1 = 0
    SPRAY_2 = 1


class Encoder:
    def id(self) -> str:
        """Identifier of this encoder"""
        pass

    def encode(self, state) -> str:
        """Encode given state into string"""
        pass


class Action:
    pass


class Emit(Action):
    def __init__(self, spray: Spray, duration: int):
        self.spray = spray
        self.duration = duration


class Wait(Action):
    def __init__(self, duration: int):
        self.duration = duration


class State:
    def __init__(self, encoder: Encoder):
        self.actions = []
        self.encoder = encoder

    def emit(self, spray: Spray, duration: int):
        self.actions.append(Emit(spray, duration))

    def wait(self, duration: int):
        self.actions.append(Wait(duration))

    def execute(self):
        print(self.encoder.encode(self))


class ByteEncoder(Encoder):
    def id(self) -> str:
        return 'byte_encoder'

    def encode(self, state: State) -> str:
        output = self.id() + '\n'
        for act in state.actions:
            output += self._encode_action(act) + '\n'

        return output + 'end_delimiter'

    def _encode_action(self, act: Action) -> str:
        if isinstance(act, Emit):
            return 'emitting the alcohol for {} duration from spray {}'.format(act.duration, act.spray)
        elif isinstance(act, Wait):
            return 'waiting for {} duration'.format(act.duration)
        else:
            return 'unimplemented action {} for {} encoder'.format(act.__class__, self.__class__)


def run():
    spray_duration = 100  # ms
    pause_duration = 200  # ms

    state = State(ByteEncoder())
    state.emit(Spray.SPRAY_1, spray_duration)
    state.wait(pause_duration)
    state.execute()


if __name__ == '__main__':
    run()
