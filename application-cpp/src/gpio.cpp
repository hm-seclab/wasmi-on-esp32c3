#include "gpio.h"

// ---------------------------
//  Pin implementation
// ---------------------------

int Pin::init(bool is_input) {
    is_initialized = true;
    return gpio_init(port, pin, is_input);
}

// ---------------------------
//  InputPin implementation
// ---------------------------

int InputPin::is_high(bool* result) {
    if (!is_initialized) {
        init(true);
    }
    unsigned int value;
    auto res = gpio_read(get_port(), get_pin(), &value);

    *result = value == 1;

    return res;
}

int InputPin::is_low(bool* result) {
    if (!is_initialized) {
        init(true);
    }
    unsigned int value;
    auto res = gpio_read(get_port(), get_pin(), &value);

    *result = value == 0;

    return res;
}

// ---------------------------
//  OutputPin implementation
// ---------------------------

int OutputPin::set_high() {
    if (!is_initialized) {
        init(false);
    }
    return gpio_write(get_port(), get_pin(), 1);
}

int OutputPin::set_low() {
    if (!is_initialized) {
        init(false);
    }
    return gpio_write(get_port(), get_pin(), 0);
}
