-- Your SQL goes here
create table roles
(
    id   integer PRIMARY KEY NOT NULL,
    name varchar(255)        NOT NULL UNIQUE
);

create table users
(
    id         integer PRIMARY KEY NOT NULL,
    first_name varchar(255)        NOT NULL,
    last_name  varchar(255)        NOT NULL,
    email      varchar(255) UNIQUE NOT NULL,
    password   varchar(88)         NOT NULL,
    status     varchar(11)         NOT NULL DEFAULT 'NotVerified' CHECK ( status in ('NotVerified', 'Verified', 'Banned') ),
    role_id    integer             NOT NULL,
    CONSTRAINT user_role FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE NO ACTION ON UPDATE NO ACTION
)