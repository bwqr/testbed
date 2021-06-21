# Frontend

This project is the place where user interacts with system.

## Preparation

In order to run this project, you will need to have nodejs bundled with npm installed on your system. You can checkout
official [nodejs](https://nodejs.org/) page for more information about installation. In order to install project's
dependencies, run the command below in this directory:

```bash
npm install
```

## Configuration

You should configure the application according to your need. Lets look at the configurations for frontend

```src/environments/environment.ts```

* production: This field is specific to angular development.
* wsEndpoint: This environment specifies the websocket connection url which is used by frontend. It should be directed to
  backend address.
* apiEndpoint: specifies the address of [backend](https://github.com/nanonetworking/kr-testbed-api/tree/master/backend)

## Running

To start a live server and directly use application on a browser, you can type

```bash
npm run start
```

Upon completion of build, you can browse **http://127.0.0.1:4100** and see the application. If you want a functional
app, you must be running [backend](https://github.com/nanonetworking/kr-testbed-api/tree/master/backend) already.
