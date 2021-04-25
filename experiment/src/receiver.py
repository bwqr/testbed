import socket
import threading
from random import random
from typing import Union

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
    def __init__(self, device_path):
        self.connection = Connection(name='connection-thread')
        self.connection.start()
        self.dev = Serial(device_path, 9600)
        self.dev.read(len(start_message))

    def next(self) -> Union[int, None]:
        if is_experiment_ended:
            if self.dev:
                self.dev.close()
                self.dev = None

            return None
        else:
            return int(self.dev.read())


def run():
    receiver = Receiver('/dev/ttyUSB0')

    rx = receiver.next()
    while rx is not None:
        for i in range(0, int(random()) * 10):
            rx = rx * 5
        print(rx)
        rx = receiver.next()


if __name__ == '__main__':
    run()
