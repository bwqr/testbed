# Experiment

This project enables user to control Testbed via a Python library.

## Preparation

If you want to just learn the library public API, you can skip this part and just go to the example part.

If you want to run this project locally, you should have a python3 interpreter installed on your system and available in
the path, then should have packages specified in the ```requirements.txt``` file installed. You can install the packages
by typing

```
pip install -r requirements.txt
```

After that point, the environment variable **PYTHON_LIB_PATH** in the ```controller/.env``` should point to the **src**
directory. You also need to copy **pyserial** lib into **src** directory. You can find **pyserial** in python site-packages
directory as **serial** is lib's directory name.

## Example

You may need to read the **API** part in order to understand these examples deeply.

In the frontend part, you can create experiment and write a Python code that describes the experiment. You can both
control the transmitter and receiver. You are expected to write one piece of Python code that handles both transmitter
part and receiver part. In one piece of code, you can differentiate that if you are executing the transmitter code or
receiver code by looking program arguments. If second argument ```sys.argv[1]``` equals to ```--transmitter```, you are
executing the transmitter part. if it equals to ```--receiver```, you are executing the receiver part.

In the receiver part, you are also given receiver devices paths. You should pass these values to Receiver class.
Whatever you print in the receiver part will be available to you at the output section of the experiment.

In the transmitter part, ```state.execute()``` statement serialize the **State** into string and writes it to the
standard output. Anything that is written to the standard output is captured by controller and sent directly to the
testbed. If you print anything in the transmitter part other than ```state.execute()``` statement does, you will likely
get error while sending commands to testbed.

Here is an example

```python
import sys
from receiver import Receiver
from transmitter import State, WordEncoder, Spray


def run_transmitter():
    spray_duration = 500  # ms
    pause_duration = 500  # ms

    state = State(WordEncoder())
    for i in range(0, 25):
        state.emit([Spray.Spray_1], spray_duration)
        state.wait(pause_duration)
    state.execute()


def run_receiver(device_paths):
    receiver = Receiver(device_paths, 5)
    ended, rx = receiver.next()
    while not ended:
        print(rx, end='')
        ended, rx = receiver.next()


if __name__ == '__main__':
    if sys.argv[1] == '--receiver':  # We know that this code handles receiver part right now 
        run_receiver(sys.argv[2:])  # don't forget to capture receiver devices paths 
    else:  # Now the code handles the transmitter part
        run_transmitter()
```

## API

### Transmitter

This part describes the transmitter module in this project

* Spray

Spray is an enum and used for differentiating the sprays in the testbed

|Name| Value |
--- | ---
|Spray_1|0|
|Spray_2|1|

* Command

Command is an abstract class that represents anything that can be executed on the testbed.

* Emit

Emit extends the Command class and represents the alcohol emittances from the sprays.

|Property |Type  | Description|
--- | --- | ---
|sprays|List[Spray]|contains the sprays that should emit the alcohol|
|duration|int|emittance duration in milliseconds|

|Method |Arguments| Return| Description|
--- | --- | --- | ---
|\_\_init\_\_|self, sprays: List[Spray], duration: int |Emit|constructor of Emit|

* Wait

Wait extends the Command class and can be used to stop alcohol emittances from the sprays.

|Property |Type  | Description|
--- | --- | ---
|duration|int|wait duration in milliseconds|

|Method |Arguments| Return| Description|
--- | --- | --- | ---
|\_\_init\_\_|self, duration: int |Wait|constructor of Wait|

* Encoder

Encoder is an abstract class and used to serialize the State into string

* WordEncoder

|Method |Arguments| Return| Description|
--- | --- | --- | ---
|encode|self, state: State |str|serializes the given state into string|

WordEncoder extends Encoder and serializes the State into string with specific format.

* State

State stores the commands to execute on the testbed. In order to send a command to testbed you should use the specific
method defined on the State.

|Property |Type  | Description|
--- | --- | ---
|commands|List[Command]|commands to be executed on the testbed|
|encoder|Encoder|the encoder that will be used while executing the state|

|Method |Arguments| Return| Description|
--- | --- | --- | ---
|\_\_init\_\_|self, encoder: Encoder |State|constructor of State|
|emit|self, sprays: List[Spray], duration: int |None|adds Emit command to the commands|
|wait|self, duration: int |None|adds Wait command to the commands|
|execute|self|None|executes all the commands that are added up to this point|

### Receiver

This part describes the receiver module in this project. In this module, there is only one class that you can use.

* Receiver

Receiver helps you to read data from receiver devices in the testbed.

|Method |Arguments| Return| Description|
--- | --- | --- | ---
|\_\_init\_\_|self, device_paths: List[str], sample_frequency: float |Receiver|Constructor of Receiver.|
|next|self|Tuple[bool, List[str]]|A tuple with first element indicates if experiment is ended, second element gives the values read from receivers|

In the constructor of Receiver, the first argument is list of paths that includes the serial path of receivers. An
example path of serial device can be ```/dev/ttyUSB0```. In the ```next``` function call, returned **List** length is
equal to the **device_paths**
length. The second argument is sample frequency, number of samples that should be read from receivers in one second. If
you give a value of ```5```, a call to next function will be returned in **200** milliseconds. A value less than or
equal to zero will cause a ```ValueError``` exception.

```next``` function returns a two elements tuple. First element indicates if experiment ended or not, meaning all the
commands in the transmitter part are executed. First time the experiment ended value returned True, you have 10 seconds
to exit the program. If you exceed 10 seconds time limit, your program will be killed.


