# Backend - Controller

This project is the backbone of system.

## Preparation

Required packages

* Rust Toolchain
* Cargo
* Postgresql
* Diesel
* Docker

### Rust - Cargo

In order to bring the backend up, you should have a rust toolchain and cargo package manager installed on your system.
You can checkout [rustup](https://rustup.rs/) for how to install a rust toolchain. After successfully installing rust
and adding it to your PATH, we can install dependencies.

### Postgresql - Diesel

To create a persistent storage, **backend** uses postgresql. You can go
to [https://www.postgresql.org/download/](https://www.postgresql.org/download/) page and download appropriate package
for your system. After installing postgresql, you should create a database and a user.

To manage the database migrations, we need to install ```diesel_cli```. If you want to migrate database manually you can
skip this part. However, before running the application, you must be sure that all queries inside
the ```migrations/**/up.sql``` files are executed. We can install ```diesel_cli``` by typing

```
cargo install diesel_cli --no-default-features --features postgres
```

In order to run migrations, type in this directory

```
diesel migration run --database-url=postgres://<username>:<password>@localhost/<database>
```

where **username**, **password** and **database** should be filled by you according to your postgresql setup.

### Docker

In order to securely execute user's code in our environment, we are using container technology to isolate execution from
host. If you want to test code execution, you will need to have docker installed on your system. You can checkout
official [docker](https://www.docker.com/) page.

## Configuration

There are two files for configuration ```testbed/.env``` and ```api/.env```. Since these files are system specific, they
do not exist in the repo. Hence, you will need to copy ```testbed/.env.example``` to ```testbed/.env```
and ```api/env.example``` to ```api/.env``` in order to obtaion configuration files. You should configure **
DATABASE_URL** inside the ```api/.env``` as we have done in diesel migration part. Another required configuration is
docker path. You must specify docker path in the ```testbed/.env``` file.

Lets specify the meanings of each entry,

```api/.env.example```

* DATABASE_URL: this environment variable specifies the database that backend connects. The format is in the form of
  ```postgres://<username>:<password>@localhost/<database>```
* RUST_LOG: specify the log level of application.
* ENV: specify the development environment.
* APP_BIND_ADDRESS: the address server listens tcp connections.
* ALLOWED_ORIGIN: specifies the origin where the requests come to the server. This enables CORS for this origin.
* WEB_APP_URL: web application url
* SECRET_KEY: This is application's secret key. It is used for cryptographic operations.
* STORAGE_PATH: specifies the storage path of application.

You do not need to change anything other than DATABASE_URL environment variable.

```testbed/.env.example```

* RUST_LOG: specify the log level of application.
* SERVER_URL: The websocket connection url of the backend, Testbed connects over this url.
* DOCKER_PATH: path to the docker executable.
* TRANSMITTER_DEVICE_PATH: USB device path for transmitter device
* RECEIVER_DEVICE_PATHS: USB device paths for receiver devices. You can specify multiple devices by separating them with
  comma
* PYTHON_LIB_PATH: path to experiment python lib
* BACKEND_ACCESS_TOKEN: Testbed uses this token to connect to the backend.

Prior to first run, you should place appropriate values for DOCKER_PATH, TRANSMITTER_DEVICE_PATH, RECEIVER_DEVICE_PATHS
and PYTHON_LIB_PATH according to your development environment.

## Running

We need two crates to be running. For backend

```
cargo run -p api
```

for controller

```
cargo run -p controller
```

Hopefully, you can start sending requests to backend.