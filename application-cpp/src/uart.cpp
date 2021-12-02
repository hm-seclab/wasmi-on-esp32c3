#include "uart.h"

int Uart::init() {
    this->is_initialized = true;

    unsigned char uart_handle = 0;
    auto result =
        uart_init(&uart_handle, this->pins.tx.get_port(),
                  this->pins.tx.get_pin(), this->pins.rx.get_port(),
                  this->pins.rx.get_pin(), nullptr, nullptr, nullptr, nullptr);

    this->handle = uart_handle;

    return result;
}

int Uart::write(unsigned char word) {
    int result = 0;
    if (!this->is_initialized) {
        result = this->init();
    }

    if (result != 0) {
        return result;
    }

    return uart_write(this->handle, word);
}

int Uart::read(unsigned char* value) {
    int result = 0;
    if (!this->is_initialized) {
        result = this->init();
    }
    if (result != 0) {
        return result;
    }

    return uart_read(this->handle, value);
}
