from random import random
from threading import Thread
from typing import Union
import socket
from serial import Serial

start_message = 'arduino_available'
end_of_experiment = 'end_of_experiment'
is_experiment_ended = False


class Receiver:
    def __init__(self, device_path):
        socket_thread = Thread(target=Receiver.__listen_event, name='socket-thread')
        socket_thread.start()
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

    @staticmethod
    def __listen_event():
        # received = None
        #
        # while received != 'end_of_experiment':
        #     received = input()
        #
        # global is_experiment_ended
        # is_experiment_ended = True

        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.bind(('0.0.0.0', 8011))
            sock.listen(1)

            global is_experiment_ended
            conn, addr = sock.accept()
            with conn:
                print('Connected by', addr)
                conn.recv(len(end_of_experiment))
                is_experiment_ended = True


def run():
    receiver = Receiver()

    rx = receiver.next()
    while rx is not None:
        for i in range(0, int(random()) * 10):
            rx = rx * 5
        print(rx)
        rx = receiver.next()


if __name__ == '__main__':
    run()
