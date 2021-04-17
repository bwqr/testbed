enum Spray {
  Spray_1,
  Spray_2
};

struct Command {
  virtual void run() const = 0;
};

struct Wait : Command {
  int duration;

  void run() const override {
    delay(this->duration);
  }
};

struct Emit : Command {
  Spray spray;
  int duration;

  void run() const override {
      digitalWrite(LED_BUILTIN, HIGH);
      delay(this->duration);
      digitalWrite(LED_BUILTIN, LOW);
  }
};

struct SetFanRPM : Command {
    int rpm;

    void run() const override {
    }
};

#define MAX_NUM_COMMAND 200

struct State {
    Command * next() {
      if (this->current_command_index < this->len) {
        auto clone = this->current_command_index;
        this->current_command_index += 1;
        return this->commands[clone];
      } else {
        return nullptr;
      }
    }

    void addCommand(Command *command) {
        if (this->len < MAX_NUM_COMMAND) {
            this->commands[this->len] = command;
            this->len += 1;
        }
    }

    void reset() {
        for (int i = 0; i < this->len; i++) {
            delete this->commands[i];
        }

        this->current_command_index = 0;
        this->len = 0;
    }

private:
    Command *commands[MAX_NUM_COMMAND];
    int current_command_index;
    int len;
};

#undef  MAX_NUM_COMMAND


enum DecodingCommand {
    Emit,
    Wait,
    SetFanRPM,
    None
};

enum DecodeResult {
    NeedMoreInput,
    UnknownCommand,
    Success
};

struct DecodeReturn {
    DecodeResult result;
    Command *command;
};

struct StreamDecoder {

    StreamDecoder() {
      this->decodingCommand = None;
      this->stage = 0;
      this->command = nullptr;
    }
  
    DecodeReturn decode(String &line) {
        if (decodingCommand == None) {
            if (line == "emit") {
                this->decodingCommand = Emit;
                this->command = new struct Emit();
            } else if (line == "wait") {
                this->decodingCommand = Wait;
                this->command = new struct Wait();
            } else if (line == "fan") {
                this->decodingCommand = SetFanRPM;
                this->command = new struct SetFanRPM();
            } else {
                // unknown command
                return {UnknownCommand, nullptr};
            }

            return {NeedMoreInput, nullptr};
        } else if (decodingCommand == Emit) {
            if (this->stage == 0) {
                static_cast<struct Emit *>(this->command)->spray = static_cast<enum Spray>(atoi(line.c_str()));
                this->stage += 1;

                return {NeedMoreInput, nullptr};
            } else {
                static_cast<struct Emit *>(this->command)->duration = atoi(line.c_str());
                auto cmd = this->getCommandAndReset();
                return {Success, cmd};
            }
        } else if (decodingCommand == Wait) {
            static_cast<struct Wait *>(this->command)->duration = atoi(line.c_str());
            auto cmd = this->getCommandAndReset();
            return {Success, cmd};
        } else if (decodingCommand == SetFanRPM) {
            static_cast<struct SetFanRPM *>(this->command)->rpm = atoi(line.c_str());
            auto cmd = this->getCommandAndReset();
            return {Success, cmd};
        }
    }

    void reset() {
      this->decodingCommand = None;
      this->stage = 0;
      if (this->command) {
        delete this->command;
      }
    }

private:
    DecodingCommand decodingCommand;
    int stage;
    Command *command;

    Command *getCommandAndReset() {
        auto clone = this->command;
        this->command = nullptr;
        this->decodingCommand = None;
        this->stage = 0;
        return clone;
    }
};

namespace outgoing {
    const char * setupMessage = "arduino_available";
    const char * endOfExperiment = "end_of_experiment";  
}

namespace incoming {
    const char * startDelimiter = "start_delimiter";
    const char * endDelimiter = "end_delimiter";
    const char * abortExperiment = "abort_experiment";  
}


bool beginDecoding = false;
bool lineCompleted = false;
bool runExperiment = false;
String line;
State state = {};
StreamDecoder decoder = {};

void setup() {
  Serial.begin(9600);

  pinMode(LED_BUILTIN, OUTPUT);

  Serial.print(outgoing::setupMessage);
}

void loop() {
    if (lineCompleted) {
        if (line == incoming::abortExperiment) {
            reset();
            return;
        }
        
        if (!beginDecoding && line == incoming::startDelimiter) {
            beginDecoding = true;
        }

        if (beginDecoding && line == incoming::endDelimiter) {
            beginDecoding = false;
            runExperiment = true;
            decoder.reset();
        }

        if (beginDecoding) {
            auto ret = decoder.decode(line);
            if (ret.result == Success) {
                state.addCommand(ret.command);
            }
        }

        lineCompleted = false;
        line = "";
    }

    if (runExperiment) {
        auto command = state.next();

        if (command != nullptr) {
          command->run(); 
        } else { // end of experiment
          Serial.print(outgoing::endOfExperiment);
          reset();
          return;
        }
    }
}

void reset() {
    state.reset();
    decoder.reset();
    beginDecoding = false;
    runExperiment = false;
    lineCompleted = false;
    line = "";
}

void serialEvent() {
  while (Serial.available()) {
    char inChar = (char) Serial.read();

    if (inChar == '\n') {
      lineCompleted = true;
    } else {
     line += inChar; 
    }
  }
}
