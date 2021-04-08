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
    void execute() {
        for (int i = 0; i < this->len; i++) {
            this->commands[i]->run();
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

        this->len = 0;
    }

private:
    Command *commands[MAX_NUM_COMMAND];
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

const char * startDelimiter = "start_delimiter";
const char *endDelimiter = "end_delimiter";

bool beginDecoding = false;
bool lineCompleted = false;
bool runExperiment = false;
String line;
State state = {};
StreamDecoder decoder = {};

void setup() {
  Serial.begin(9600);

  pinMode(LED_BUILTIN, OUTPUT);
}

void loop() {
    if (lineCompleted) {
        if (!beginDecoding && line == startDelimiter) {
            beginDecoding = true;
        }

        if (beginDecoding && line == endDelimiter) {
            beginDecoding = false;
            runExperiment = true;
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
        state.execute();
        state.reset();
        runExperiment = false;
    }
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
