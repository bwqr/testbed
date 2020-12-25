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

insert into users (id, first_name, last_name, email, password, status, role_id)
values (1, 'My FirstName', 'My LastName', 'hola@email.com',
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

create table runners
(
    id         serial PRIMARY KEY  NOT NULL,
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

insert into runners (access_key)
values ('runner_1')