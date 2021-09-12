import socket
import threading
import time
from typing import Tuple, List

from serial import Serial

from log import debug

class Outgoing:
    class Tcp:
        start_of_experiment = "start_of_experiment"

class Incoming:
    class Tcp:
        end_of_experiment = 'end_of_experiment'


is_experiment_ended = False


class Connection(threading.Thread):
    def __init__(self, *args, **kwargs):
        self.__socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.__socket.bind(('0.0.0.0', 8011))
        self.__socket.listen(1)
        self.__socket.settimeout(1)

        while True:
            try:
                debug('accepting new connection')

                conn, _ = self.__socket.accept()

                debug('a connection is accepted')

                conn.send(Outgoing.Tcp.start_of_experiment.encode('utf-8'))
                conn.close()

                break
            except socket.timeout:
                debug('failed to accept connection, time out')

        super(Connection, self).__init__(*args, **kwargs)

    def run(self) -> None:
        global is_experiment_ended

        # run until main thread terminates or Connection thread receives a connection
        while threading.main_thread().is_alive():
            try:
                debug('waiting for the end of experiment')

                conn, _ = self.__socket.accept()
                conn.recv(len(Incoming.Tcp.end_of_experiment))

                is_experiment_ended = True

                debug('received end of experiment')

                break
            except socket.timeout:
                pass

        self.__socket.close()


class Receiver:
    def __init__(self, device_paths: List[str], sample_frequency):
        if sample_frequency <= 0:
            raise ValueError('sample rate cannot be less than or equal to zero')

        self.__sample_frequency = sample_frequency

        self.__devs = list(map(lambda path: Serial(path, 9600), device_paths))

        self.__connection = Connection(name='connection-thread')
        self.__connection.start()

    def next(self) -> Tuple[bool, List[int]]:
        read_data = []
        start_time = time.monotonic_ns()

        for dev in self.__devs:
            dev.flushInput()
            result = b''

            while dev.read() != b' ':
                pass
            dev.read()  # \r
            dev.read()  # \n

            val = dev.read()
            while val != b' ':
                result += val
                val = dev.read()

            read_data.append(result.decode('utf-8'))

        # in seconds
        elapsed_time = (time.monotonic_ns() - start_time) / 1_000_000_000
        sample_time = 1 / self.__sample_frequency
        # in seconds
        sleep_time = sample_time - elapsed_time

        if sleep_time > 0:
            time.sleep(sleep_time)
        else:
            # receiver could not keep up with sampling rate
            debug('receiver could not keep up with sampling rate, time difference {}'.format(-sleep_time))

        return is_experiment_ended, read_data


def run():
    receiver = Receiver(['/dev/ttyUSB1'], 0.2)

    ended, data = receiver.next()
    while not ended:
        print(data)

        ended, data = receiver.next()


if __name__ == '__main__':
    run()
