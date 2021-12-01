#include "uart.h"

int init_uart(struct uart* uart) {
    unsigned char handle;
    
    int res = uart_init(&handle, uart->tx.port, uart->tx.pin, uart->rx.port, uart->rx.pin, NULL, NULL, NULL, NULL);
    uart->handle = handle;

    return res;
}

int write(const struct uart* uart, unsigned char value) {
    return uart_write(uart->handle, value);
}

int read(const struct uart* uart, unsigned char* value) {
    return uart_read(uart->handle, value);
}
