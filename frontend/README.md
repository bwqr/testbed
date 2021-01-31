# Frontend

This project is the place where user interacts with system.

## Preparetation
In order to run this project, you will need to have nodejs bundled with npm installed on your system. You can checkout official [nodejs](https://nodejs.org/) page for more information about installation. In order to install project's dependencies, run the command below in this directory:
```bash
npm install
```

## Configuration

You should configure the application according to your need. You can specify the backend endpoint in the file ```src/environments/environment.ts```. The key ```apiEndpoint``` should be directing to your [backend](https://github.com/nanonetworking/kr-testbed-api/tree/master/backend).

## Running

To start a live server and directly use application on a browser, you can type 
```bash
npm run start
```
Upon completion of build, you can browse **http://127.0.0.1:4100** and see the application. If you want a functional app, you must be running [backend](https://github.com/nanonetworking/kr-testbed-api/tree/master/backend) already.