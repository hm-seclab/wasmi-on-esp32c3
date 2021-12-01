#include "gpio.h"
#include "uart.h"

#define CHECK_ERROR(expr) \
    {                     \
        int res = expr;   \
        if (res != 0) {   \
            return res;   \
        }                 \
    }

// Sets up a uart connection over the pins
// 2 (rx) and 3 (tx).
inline int setup_uart(struct uart* uart) {
    struct input_pin pin_2;
    pin_2.port = 0;
    pin_2.pin = 2;
    gpio_input_init(&pin_2);

    struct output_pin pin_3;
    pin_3.port = 0;
    pin_3.pin = 3;
    gpio_output_init(&pin_3);

    uart->rx = pin_2;
    uart->tx = pin_3;

    return init_uart(uart);
}

// the main function that is exported by the module.
int start() {
    // setup pin 10 as input
    struct input_pin pin_10;
    pin_10.port = 0;
    pin_10.pin = 10;
    CHECK_ERROR(gpio_input_init(&pin_10));

    // setup pin 8 (LED) as output
    struct output_pin pin_8;
    pin_8.port = 0;
    pin_8.pin = 8;
    CHECK_ERROR(gpio_output_init(&pin_8));

    // setup a uart connection
    struct uart uart;
    CHECK_ERROR(setup_uart(&uart));
    while (1) {
        // turn the LED on
        CHECK_ERROR(set_high(&pin_8));

        // read the value of pin 10 and
        // write it to the console and over uart
        bool val;
        CHECK_ERROR(is_high(&pin_10, &val));
        if (val) {
            print("val_10 is hi", 13);
        } else {
            print("val_10 is lo", 13);
        }
        write(&uart, (unsigned char)val);

        // wait for a second
        delay_ms(1000);

        // turn the led off
        CHECK_ERROR(set_low(&pin_8));
        
        // wait for another second
        delay_ms(1000);
    }
}
