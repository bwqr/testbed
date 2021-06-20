CREATE FUNCTION update_timestamp() RETURNS TRIGGER
    LANGUAGE plpgsql
AS
$$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$;

-- Your SQL goes here
create table roles
(
    id   serial PRIMARY KEY NOT NULL,
    name varchar(255)       NOT NULL UNIQUE
);

insert into roles (name)
values ('admin'),
       ('user');

create table users
(
    id         serial PRIMARY KEY  NOT NULL,
    first_name varchar(122)        NOT NULL,
    last_name  varchar(122)        NOT NULL,
    email      varchar(255) UNIQUE NOT NULL,
    password   varchar(88)         NOT NULL,
    status     varchar(11)         NOT NULL DEFAULT 'NotVerified' CHECK ( status in ('NotVerified', 'Verified', 'Banned') ),
    role_id    integer             NOT NULL,
    CONSTRAINT user_role_id FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE NO ACTION ON UPDATE NO ACTION
);

insert into users (first_name, last_name, email, password, status, role_id)
values ('My FirstName', 'My LastName', 'hola@email.com',
        'KzOo58lsjoE3cWBEDuYlh5/4b1SxQMezrHt7UqM2H+xI/YwdOGq7SzDqFp6uA0YrPc9l5x9qRoGeMJklcQWinw==', 'Verified', 2);

create table experiments
(
    id         serial PRIMARY KEY NOT NULL,
    user_id    integer            NOT NULL,
    name       varchar(255)       NOT NULL,
    code       text               NOT NULL DEFAULT '',
    created_at timestamp          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT experiment_user_id FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE NO ACTION ON UPDATE NO ACTION
);

create trigger experiments_updated_at
    before update
    on experiments
    for each row
execute procedure update_timestamp();

create table runners
(
    id         serial PRIMARY KEY  NOT NULL,
    name       varchar(256)        NOT NULL,
    access_key varchar(191) UNIQUE NOT NULL,
    created_at timestamp           NOT NULL DEFAULT CURRENT_TIMESTAMP
);

create table jobs
(
    id            serial PRIMARY KEY NOT NULL,
    experiment_id integer            NOT NULL,
    runner_id     integer            NOT NULL,
    code          text               NOT NULL,
    status        varchar(11)        NOT NULL DEFAULT 'Pending' CHECK ( status in ('Pending', 'Running', 'Successful', 'Failed') ),
    created_at    timestamp          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    timestamp          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT job_experiment_id FOREIGN KEY (experiment_id) REFERENCES experiments (id) ON DELETE CASCADE ON UPDATE NO ACTION,
    CONSTRAINT job_runner_id FOREIGN KEY (runner_id) REFERENCES runners (id) ON DELETE NO ACTION ON UPDATE NO ACTION
);

create trigger jobs_updated_at
    before update
    on jobs
    for each row
execute procedure update_timestamp();

insert into runners (name, access_key)
values ('NanoNetworking Testbed', 'runner_1');

INSERT INTO public.experiments (user_id, name, code, created_at, updated_at)
VALUES (1, 'transmitter crash', 'import sys
from random import random
from receiver import Receiver
from transmitter import State, WordEncoder, Spray

def run_transmitter():
    raise Exception

def run_receiver(device_paths):
    pass

if __name__ == ''__main__'':
    if sys.argv[1] == ''--receiver'':
        run_receiver(sys.argv[2:])
    else:
        run_transmitter()', '2021-04-25 11:58:41.265656', '2021-04-25 11:58:41.265656');
INSERT INTO public.experiments (user_id, name, code, created_at, updated_at)
VALUES (1, 'transmitter ddos', 'import sys
from random import random
from receiver import Receiver
from transmitter import State, WordEncoder, Spray
import time

def run_transmitter():
    time.sleep(15)


def run_receiver(device_paths):
    pass

if __name__ == ''__main__'':
    if sys.argv[1] == ''--receiver'':
        run_receiver(sys.argv[2:])
    else:
        run_transmitter()', '2021-04-18 16:09:36.646962', '2021-04-18 16:09:36.646962');
INSERT INTO public.experiments (user_id, name, code, created_at, updated_at)
VALUES (1, 'output xss hacking', 'import sys
from random import random
from receiver import Receiver
from transmitter import State, WordEncoder, Spray


def run_transmitter():
    state = State(WordEncoder())
    state.execute()
    print("""
      <script>alert(''Hacked'')</script>
  """)
    raise Exception


def run_receiver(device_paths):
    pass


if __name__ == ''__main__'':
    if sys.argv[1] == ''--receiver'':
        run_receiver(sys.argv[2:])
    else:
        run_transmitter()', '2021-04-18 16:04:44.018911', '2021-04-18 16:04:44.018911');
INSERT INTO public.experiments (user_id, name, code, created_at, updated_at)
VALUES (1, 'receiver crash', 'import sys
from random import random
from receiver import Receiver
from transmitter import State, WordEncoder, Spray

def run_transmitter():
    state = State(WordEncoder())
    state.wait(5000)
    state.execute()


def run_receiver(device_paths):
    raise Exception


if __name__ == ''__main__'':
    if sys.argv[1] == ''--receiver'':
        run_receiver(sys.argv[2:])
    else:
        run_transmitter()', '2021-04-25 11:58:01.943044', '2021-04-25 11:58:01.943044');
INSERT INTO public.experiments (user_id, name, code, created_at, updated_at)
VALUES (1, 'receiver exit without waiting experiment end', 'import sys
from random import random
from receiver import Receiver
from transmitter import State, WordEncoder, Spray

def run_transmitter():
    spray_duration = 20  # ms
    pause_duration = 25  # ms

    state = State(WordEncoder())
    for i in range(0, 25):
        state.emit([Spray.Spray_1], spray_duration * i)
        state.wait(pause_duration * i)

    state.execute()


def run_receiver(device_paths):
    return


if __name__ == ''__main__'':
    if sys.argv[1] == ''--receiver'':
        run_receiver(sys.argv[2:])
    else:
        run_transmitter()', '2021-04-25 11:54:01.737986', '2021-04-25 11:54:01.737986');
INSERT INTO public.experiments (user_id, name, code, created_at, updated_at)
VALUES (1, 'normal', 'import sys
from random import random
from receiver import Receiver
from transmitter import State, WordEncoder, Spray


def run_transmitter():
    spray_duration = 20  # ms
    pause_duration = 25  # ms

    state = State(WordEncoder())
    for i in range(0, 25):
        state.emit([Spray.Spray_1], spray_duration * i)
        state.wait(pause_duration)
    state.execute()


def run_receiver(device_paths):
    receiver = Receiver(device_paths)
    ended, rx = receiver.next()
    while not ended:
       print(rx)
       ended, rx = receiver.next()

if __name__ == &#x27;__main__&#x27;:
    if sys.argv[1] == &#x27;--receiver&#x27;:
        run_receiver(sys.argv[2:])
    else:
        run_transmitter()', '2021-03-24 13:16:35.128303', '2021-03-24 13:16:35.128303');
INSERT INTO public.experiments (user_id, name, code, created_at, updated_at)
VALUES (1, 'receiver ddos', 'import sys
from random import random
from receiver import Receiver
from transmitter import State, WordEncoder, Spray
import time

def run_transmitter():
    state = State(WordEncoder())
    state.wait(5000)
    state.execute()


def run_receiver(device_paths):
    receiver = Receiver(device_paths)
    time.sleep(20)


if __name__ == ''__main__'':
    if sys.argv[1] == ''--receiver'':
        run_receiver(sys.argv[2:])
    else:
        run_transmitter()', '2021-04-25 11:52:50.821539', '2021-04-25 11:52:50.821539');

create table slots
(
    id         serial PRIMARY KEY NOT NULL,
    user_id    integer            NOT NULL,
    runner_id  integer            NOT NULL,
    start_at   timestamp          NOT NULL,
    end_at     timestamp          NOT NULL,
    created_at timestamp          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT slot_user_id FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE ON UPDATE NO ACTION,
    CONSTRAINT slot_runner_id FOREIGN KEY (runner_id) REFERENCES runners (id) ON DELETE NO ACTION ON UPDATE NO ACTION
);

create trigger slots_updated_at
    before update
    on slots
    for each row
execute procedure update_timestamp();