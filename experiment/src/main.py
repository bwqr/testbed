import sys
from typing import List

from receiver import Receiver
from transmitter import State, WordEncoder, Spray


def run_transmitter():
    spray_duration = 100  # ms
    pause_duration = 200  # ms

    state = State(WordEncoder())
    state.wait(3000)
    state.emit([Spray.Spray_1], 2500)
    for i in range(0, 20):
        state.emit([Spray.Spray_1], spray_duration)
        state.wait(pause_duration)

    state.execute()


def run_receiver(device_paths: List[str]):
    receiver = Receiver(device_paths)

    ended, rx = receiver.next()
    while ended is not None:
        for d in rx:
            print(d)

        ended, data = receiver.next()


if __name__ == '__main__':
    if sys.argv[1] == '--receiver':
        run_receiver(sys.argv[2:])
    else:
        run_transmitter()
