#include "gpio.h"

int gpio_input_init(struct input_pin* ip) {
    return gpio_init(ip->port, ip->pin, true);
}

int gpio_output_init(struct output_pin* op) {
    return gpio_init(op->port, op->pin, false);
}

int set_high(struct output_pin* op) { return gpio_write(op->port, op->pin, 1); }

int set_low(struct output_pin* op) { return gpio_write(op->port, op->pin, 0); }

int is_low(const struct input_pin* ip, bool* location_of) {
    unsigned int value;

    int res = gpio_read(ip->port, ip->pin, &value);
    if (res != 0) {
        return res;
    }

    *location_of = value == 0;

    return res;
}

int is_high(const struct input_pin* ip, bool* location_of) {
    unsigned int value;

    int res = gpio_read(ip->port, ip->pin, &value);
    if (res != 0) {
        return res;
    }

    *location_of = value == 1;

    return res;
}
