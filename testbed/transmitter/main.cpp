#include <iostream>
#include <vector>
#include <memory>
#include <string>

const char * start_delimiter = "start_delimiter";
const char * end_delimiter = "end_delimiter";

enum Spray {
    Spray_1,
    Spray_2
};

enum Decoders {
    WordDecoder
};

struct Action {
    virtual void act() const = 0;
};

struct Emit : Action {
    Spray spray;
    int duration;

    void act() const override {
        std::cout << "emitting the alcohol for " << this->duration << " duration from spray " << this->spray << std::endl;
    }
};

struct Wait : Action {
    int duration;

    void act() const override {
        std::cout << "waiting for " << this->duration <<" duration" << std::endl;
    }
};

struct State {
    std::vector<std::unique_ptr<Action>> actions;

    void execute() {
        for (const auto &action: this->actions) {
            action->act();
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
        for (std::string line;std::getline(std::cin, line);) {
            if (line == end_delimiter) {
                break;
            }

            if (line == "emit") {
                auto p = new Emit();

                std::getline(std::cin, line);

                p->spray = static_cast<Spray>(std::stoi(line));

                std::getline(std::cin, line);

                p->duration = std::stoi(line);

                state.actions.push_back(std::unique_ptr<Action>(p));
            } else if (line == "wait") {
                auto p = new Wait();

                std::getline(std::cin, line);

                p->duration = std::stoi(line);
            
                state.actions.push_back(std::unique_ptr<Action>(p));
            } else {
                std::cout << "unknown action " << line << std::endl; 
            }
        }

        std::cout << "decoded" << std::endl;

        return state;
    }
};

std::unique_ptr<Decoder> find_decoder(int decoder_id) {
    if (decoder_id == WordDecoder::id) {
        return std::unique_ptr<Decoder>(new struct WordDecoder());
    }

    return nullptr;
}

int main(int argc, char **argv) {
    std::string line;

    // consume until eof or start_delimiter
    while(line != start_delimiter && std::getline(std::cin, line));

    std::getline(std::cin, line);

    int decoder_id = std::stoi(line.c_str());
    std::unique_ptr<Decoder> decoder = find_decoder(decoder_id);

    if (!decoder) {
        std::cout << "unknown decoder " << decoder_id << std::endl;

        return -1;
    }

    State state = decoder->decode();

    state.execute();

    return 0;
}