import socket

end_of_experiment = 'end_of_experiment'


def run():
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.connect(('127.0.0.1', 8011))
        sock.sendall(end_of_experiment.encode())
        print('sended')


if __name__ == '__main__':
    run()
