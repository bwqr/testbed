#include <iostream>
#include <vector>
#include <memory>
#include <string>

const char *start_delimiter = "start_delimiter";
const char *end_delimiter = "end_delimiter";

enum Spray {
    Spray_1,
    Spray_2
};

enum Decoders {
    WordDecoder
};

struct Command {
    virtual void run() const = 0;
};

struct Emit : Command {
    Spray spray;
    int duration;

    void run() const override {
        std::cout << "emitting the alcohol for " << this->duration << " duration from spray " << this->spray
                  << std::endl;
    }
};

struct Wait : Command {
    int duration;

    void run() const override {
        std::cout << "waiting for " << this->duration << " duration" << std::endl;
    }
};

struct SetFanRPM : Command {
    int rpm;

    void run() const override {
        std::cout << "setting fan rpm " << this->rpm << std::endl;
    }
};

struct State {
    std::vector <std::unique_ptr<Command>> commands;

    void execute() {
        for (const auto &command: this->commands) {
            command->run();
        }
    }
};

struct Decoder {
    static const int id;

    virtual State decode() const = 0;
};

struct WordDecoder : Decoder {
    static const int id = Decoders::WordDecoder;

    State decode() const override {

        State state;

        std::cout << "decoding" << std::endl;
        for (std::string line; std::getline(std::cin, line);) {
            if (line == end_delimiter) {
                break;
            }

            if (line == "emit") {
                auto p = new Emit();

                std::getline(std::cin, line);

                p->spray = static_cast<Spray>(std::stoi(line));

                std::getline(std::cin, line);

                p->duration = std::stoi(line);

                state.commands.push_back(std::unique_ptr<Command>(p));
            } else if (line == "wait") {
                auto p = new Wait();

                std::getline(std::cin, line);

                p->duration = std::stoi(line);

                state.commands.push_back(std::unique_ptr<Command>(p));
            } else if (line == "fan") {
                auto p = new SetFanRPM();

                std::getline(std::cin, line);

                p->rpm = std::stoi(line);

                state.commands.push_back(std::unique_ptr<Command>(p));
            } else {
                std::cout << "unknown action " << line << std::endl;
            }
        }

        std::cout << "decoded" << std::endl;

        return state;
    }
};

std::unique_ptr <Decoder> find_decoder(int decoder_id) {
    if (decoder_id == WordDecoder::id) {
        return std::unique_ptr<Decoder>(new struct WordDecoder());
    }

    return nullptr;
}

int main(int argc, char **argv) {
    std::string line;

    // consume until eof or start_delimiter
    while (line != start_delimiter && std::getline(std::cin, line));

    std::getline(std::cin, line);

    int decoder_id = std::stoi(line);
    std::unique_ptr <Decoder> decoder = find_decoder(decoder_id);

    if (!decoder) {
        std::cout << "unknown decoder " << decoder_id << std::endl;

        return -1;
    }

    State state = decoder->decode();

    state.execute();

    return 0;
}