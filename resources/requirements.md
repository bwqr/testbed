# Requirements
## User
* Auth
    * User shall be able to login with his/her email and password.
    * User shall be able to sign up with his/her name, surname, email and password
    * User shall be able to reset his/her account password via his/her registered email.
* Profile
    * User shall be able to change his/her name and surname.
* Settings
    * User shall be able to change his/her password.
* Experiment
    * User shall be able to edit and save his/her own code. // Check
    * User shall be able to upload his/her experiment code for execution.
    * User shall be able to list his/her all experiments, including past, running and pending experiments.
    * User shall be able to run his/her code.
    * User shall be able to see the output of execution, like summary, logs, values, etc.
*
## System
* Auth
    * System shall authenticate user with his/her email and password.
    * System shall register user with his/ her name, surname, email and password
    * System shall send a reset password mail to user's email address.
* Profile
    * System shall let user change his/her name and surname.
* Settings
    * System shall let user change his/her password.
* Experiment
    * System shall let user edit and save his/her own code.
    * System shall accept experiment codes as files.
    * System shall be able to list all experiments for a user, including past, running and pending experiements.
    * System shall be able to use Experiment component to execute user's codes.
    * System shall store user's codes in its environment.
    * System shall store output of executed codes in a semantically meaningfull way.
    * System shall have a pending executions queue.

## Experiment
* Analyzer
    * Analyzer shall be able to summarize output of execution.
* Executor
    * Execute User's experiment code in Backend.
* Backend
    * There should be an interface called Backend in order to provide a transparent API to users.
    * Backend shall be able to control the RPM of fan.
    * Backend shall be able to ...
    * Backend shall be able to compute pre defined algorithms like channel estimation, channel coding, distance between Tx and Rx, etc.
