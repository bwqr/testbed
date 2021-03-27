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


class WordEncoder(Encoder):
    id: int = int(Encoders.ByteEncoder)

    def encode(self, state: State) -> str:
        output = start_delimiter + '\n' + str(WordEncoder.id) + '\n'
        for act in state.actions:
            output += self._encode_action(act) + '\n'

        return output + 'end_delimiter'

    def _encode_action(self, act: Action) -> str:
        if isinstance(act, Emit):
            return 'emit\n{}\n{}'.format(int(act.spray), act.duration)
        elif isinstance(act, Wait):
            return 'wait\n{}'.format(int(act.duration))
        else:
            print('unimplemented action {} for {} encoder'.format(act.__class__, self.__class__))
            return ''


def run():
    spray_duration = 100  # ms
    pause_duration = 200  # ms

    state = State(WordEncoder())
    state.emit(Spray.Spray_1, spray_duration)
    state.wait(pause_duration)
    state.execute()


if __name__ == '__main__':
    run()
