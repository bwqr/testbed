# Backend - Executor
This project is the backbone of system.

## Preparetaion
Required packages
* Rust Toolchain
* Cargo
* Postgresql
* Diesel
* Docker

### Rust - Cargo

In order to bring the backend up, you should have a rust toolchain and cargo package manager installed on your system. You can checkout [rustup](https://rustup.rs/) for how to install a rust toolchain. After successfully installing rust and adding it to your PATH, we can install dependencies. 


### Postgresql - Diesel
To create a persistent storage, **backend** uses postgresql. You can go to [https://www.postgresql.org/download/](https://www.postgresql.org/download/) page and download appropriate package for your system. After installing postgresql, you should create a database and a user.

To manage the database migrations, we need to install ```diesel_cli```. If you want to migrate database manually you can skip this part. However, before running the application, you must be sure that all queries inside the ```migrations/**/up.sql``` files are executed. We can install ```diesel_cli``` by typing 
```
cargo install diesel_cli --no-default-features --features postgres
```
In order to run migrations, type in this directory
```
diesel migration run --database-url=postgres://<username>:<password>@localhost/<database>
```
where **username**, **password** and **database** should be filled by you according to your postgresql setup.

### Docker
In order to securely execute user's code in our environment, we are using container technology to isolate execution from host. If you want to test code execution, you will need to have docker installed on your system. You can checkout official [docker](https://www.docker.com/) page.

## Configuration
There are two files for configuration ```testbed/.env``` and ```api/.env```. Since these files are system specific, they do not exist in the repo. Hence, you will need to copy ```testbed/.env.example``` to ```testbed/.env``` and ```api/env.example``` to ```api/.env``` in order to obtaion configuration files. You should configure **DATABASE_URL** inside the ```api/.env``` as we have done in diesel migration part. Another required configuration is docker path. You must specify docker path in the ```testbed/.env``` file.

If you did not change any other configuration, then we are good to go.

## Running

We need two packages to be running. For backend
```
cargo run -p api
```
And for executor
```
cargo run -p testbed
```

Hopefully, you can start sending requests to backend.