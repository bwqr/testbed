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

create table experiments
(
    id         serial PRIMARY KEY NOT NULL,
    user_id    integer            NOT NULL,
    name       varchar(255)       NOT NULL,
    status     varchar(11)        NOT NULL DEFAULT 'NoAction' CHECK ( status in ('Pending', 'Running', 'NoAction', 'Failed') ),
    created_at timestamp          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT experiment_user_id FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE NO ACTION ON UPDATE NO ACTION
)