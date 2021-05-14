import sys
from typing import List

from receiver import Receiver
from transmitter import State, WordEncoder, Spray


def run_transmitter():
    spray_duration = 20  # ms
    pause_duration = 25  # ms

    state = State(WordEncoder())
    for i in range(0, 25):
        state.emit([Spray.Spray_1], spray_duration)
        state.wait(pause_duration)

    state.execute()


def run_receiver(device_paths: List[str]):
    receiver = Receiver(device_paths)
    ended, rx = receiver.next()
    while not ended:
        print(rx)
        ended, rx = receiver.next()


if __name__ == '__main__':
    if sys.argv[1] == '--receiver':
        run_receiver(sys.argv[2:])
    else:
        run_transmitter()
