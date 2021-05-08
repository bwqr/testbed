const int sprayControlPin1 = 2; // the pin number for the digital output pin contrillong the Spray1
const int sprayControlPin2 = 4; // the pin number for the digital output pin contrillong the Spray2

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
      auto sprayPin = sprayControlPin1;
      if (this->spray == Spray_2) {
        sprayPin = sprayControlPin2;
      }
      digitalWrite(sprayPin, HIGH);
      digitalWrite(LED_BUILTIN, HIGH);
      delay(this->duration);
      digitalWrite(sprayPin, LOW);
      digitalWrite(LED_BUILTIN, LOW);
  }
};

struct SetFanRPM : Command {
    int rpm;

    void run() const override {
    }
};

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
        this->command = nullptr;
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
}


bool beginDecoding = false;
bool lineCompleted = false;
String line;
StreamDecoder decoder = {};

void setup() {
  Serial.begin(9600);

  pinMode(LED_BUILTIN, OUTPUT);

  Serial.print(outgoing::setupMessage);

    // initialize the spray control pin as output:
  pinMode(sprayControlPin1, OUTPUT); 
  pinMode(sprayControlPin2, OUTPUT); 
}

void loop() {
    if (lineCompleted) {
        if (!beginDecoding && line == incoming::startDelimiter) {
            beginDecoding = true;
        }

        if (beginDecoding && line == incoming::endDelimiter) {
            beginDecoding = false;
            decoder.reset();
        }

        if (beginDecoding) {
            auto ret = decoder.decode(line);
            if (ret.result == Success) {
                ret.command->run();
                delete ret.command;
                Serial.print(outgoing::endOfExperiment);
            }
        }

        lineCompleted = false;
        line = "";
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
