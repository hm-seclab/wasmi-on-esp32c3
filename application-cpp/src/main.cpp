#include "gpio.h"
#include "uart.h"

#define CHECK_ERROR(expr) \
    {                     \
        int res = expr;   \
        if (res != 0) {   \
            return res;   \
        }                 \
    }

Uart setup_uart() {
    auto tx = OutputPin(0, 3);
    auto rx = InputPin(0, 2);
    return Uart(Pins(rx, tx, nullptr, nullptr));
}

extern "C" {
int start() {
    // init pin 8 (LED) as output
    OutputPin gpio_8 = OutputPin(0, 8);
    // init pin 10 as input
    InputPin gpio_10 = InputPin(0, 10);

    Uart uart = setup_uart();
    while (1) {
        // turn led on
        CHECK_ERROR(gpio_8.set_high());

        delay_ms(1000);

        // read gpio value from gpio 10
        bool val_10;
        CHECK_ERROR(gpio_10.is_high(&val_10));
        if (val_10) {
            print("pin 10 is hi", 13);
            uart.write((char)val_10);
        } else {
            print("pin 10 is lo", 13);
            uart.write((char)val_10);
        }
        // turn led off and delay again
        CHECK_ERROR(gpio_8.set_low());
        delay_ms(1000);
    }
}
}
