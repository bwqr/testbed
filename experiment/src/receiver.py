import socket
import threading
from random import random
from typing import Tuple, List

from serial import Serial

start_message = 'arduino_available'
end_of_experiment = 'end_of_experiment'
is_experiment_ended = False


class Connection(threading.Thread):
    def __init__(self, *args, **kwargs):
        super(Connection, self).__init__(*args, **kwargs)

    def run(self) -> None:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            global is_experiment_ended

            sock.bind(('0.0.0.0', 8011))
            sock.listen(1)
            sock.settimeout(1)
            # run until main thread terminates
            while threading.main_thread().is_alive():
                try:
                    conn, addr = sock.accept()
                    with conn:
                        conn.recv(len(end_of_experiment))
                        is_experiment_ended = True
                # socket errors are ignored, like timeout
                except socket.error:
                    pass


class Receiver:
    def __init__(self, device_paths: List[str]):
        self.connection = Connection(name='connection-thread')
        self.connection.start()
        self.devs = list(map(lambda path: Serial(path, 9600), device_paths))

    def next(self) -> Tuple[bool, List[int]]:
        read_data = []
        for dev in self.devs:
            read_data.append(int(dev.read()))

        return is_experiment_ended, read_data


def run():
    receiver = Receiver(['/dev/ttyUSB0', '/dev/ttyUSB1'])

    i = 0
    ended, data = receiver.next()
    while ended is not None and i < 5000:
        if ended:
            i += 1

        for d in data:
            for _ in range(0, int(random() * 10)):
                d = d * 5
            print(d)

        ended, data = receiver.next()


if __name__ == '__main__':
    run()
