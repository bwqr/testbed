# kr-testbed-api

This project enables an user to control nanonetworking testbed remotely by submitting his/her own code via web application.

Project consists of five parts:
* Backend
* Frontend
* Controller
* Testbed API
* Testbed Transmitter Device

### Backend
Backend is the backbone of the project. It serves data to user, dispatches jobs to Controller, etc.

### Frontend
Frontend is the place where user interacts with system.

### Controller
Controller handles user's code. It is placed inside the [backend](https://github.com/nanonetworking/kr-testbed-api/tree/master/backend) directory

### Testbed API
Testbed API provides set of functions to user in order to interact with Nanonetworking Testbed. It is placed inside the [experiment](https://github.com/nanonetworking/kr-testbed-api/tree/master/experiment)

### Testbed Transmitter Device
This is an arduino project, it is placed in the [testbed/transmitter](https://github.com/nanonetworking/kr-testbed-api/tree/master/testbed/transmitter) directory

## Running the project

Checkout [frontend](https://github.com/nanonetworking/kr-testbed-api/tree/master/frontend), [experiment](https://github.com/nanonetworking/kr-testbed-api/tree/master/experiment) and [backend](https://github.com/nanonetworking/kr-testbed-api/tree/master/backend) pages for more information about running the project.
