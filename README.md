# kr-testbed-api
In this documentation you can find
* [Description](#description)
* [Components](#components)
    * [Backend](#backend)
    * [Frontend](#frontend)
    * [Controller](#controller)
    * [Testbed API](#testbed-api)
    * [Testbed Transmitter Device](#testbed-transmitter-device)
* [Running the Project](#running-the-project)

## Description
This project enables a user to control Nanonetworking Testbed remotely by submitting his/her own code via web application. For users who want to use NanoNetworking Testbed remotely, they should check the Testbed API part of this project.

The project is mainly developed with Nanonetworking Testbed in mind. However, it can be applied to other remotely controllable experiment sets.

## Components

Project consists of five different components:
* Backend
* Frontend
* Controller
* Testbed API
* Testbed Transmitter Device

### Backend
Backend is the backbone of the project. It serves data to the user, dispatches jobs to Controller, etc.

### Frontend
Frontend is the place where the user interacts with system.

### Controller
Controller handles user's code. It is placed inside the [backend](https://github.com/nanonetworking/kr-testbed-api/tree/master/backend)

### Testbed API
Testbed API provides a Python library to the user in order to interact with Nanonetworking Testbed. This Python library provides sets of classes. It is placed inside the [experiment](https://github.com/nanonetworking/kr-testbed-api/tree/master/experiment)

### Testbed Transmitter Device
This is an arduino project which is written for Transmitter Device in the Testbed,
it is placed in the [testbed/transmitter](https://github.com/nanonetworking/kr-testbed-api/tree/master/testbed/transmitter) directory


## Running the Project
Although each component, except Testbed Transmitter Device, has its own documentation in its directory about running itself, we also explain in this section how running components can work together.

### 1. Backend
The component that should be running before other components is *backend*. By following the documentation in [backend](https://github.com/nanonetworking/kr-testbed-api/tree/master/backend), you can easily start running the *backend* component. You may need to modify some environment variables to establish communication between *frontend* and *backend* components.
### 2. Testbed API
After running *backend* successfully, you can setup the *Testbed API* component. You can find detailed documentation about setting up the component in the [Preparation section of experiment](https://github.com/nanonetworking/kr-testbed-api/tree/master/experiment#preparation).

**An important note! *Testbed API* component must be setup in the same environment of *controller* component since *controller* will use *Testbed API*.**

### 3. Testbed Transmitter Device
For this part, you just need to upload transmitter code into arduino device which is going to be use for Testbed Transmitter Device. You can open **testbed/transmitter/transmitter.ino** file with Arduino IDE and upload it to the device. Afterwards, you should setup the connections between arduino device and Nanonetworking Testbed's sprays.

### 4. Controller
Later on, you can start running *controller* as described in the [backend](https://github.com/nanonetworking/kr-testbed-api/tree/master/backend). *Controller* component is designed to run on a linux machine. This linux machine must have docker installed. The linux machine also must have Transmitter and Receiver devices connected to itself via USB interface. In order to communicate with docker and control USB devices, the user used in linux machine must have required permissions. For docker, user should be in docker group. For USB devices, either user should own files belonging to the USB devices or user should be in the group which owns the USB devices. Otherwise, you will receive *Permission denied* error while trying to run an experiment.

Note that you also need to modify some environment variables to establish communication between *backend* and *controller*, *testbed devices* and *controller*, *Testbed API* and *controller* components. Required environment variables are listed in the documentation.

### 5. Frontend
Finally, you can start running *frontend* and interact with the system. You can find detailed documentation about running *frontend* in [here](https://github.com/nanonetworking/kr-testbed-api/tree/master/frontend)
