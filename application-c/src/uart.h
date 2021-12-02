#ifndef _UART_H_
#define _UART_H_
#include "gpio.h"
#include "runtime.h"

/**
 * A structure that holds information
 * about a uart connection. This involves
 * the four uart related pins and a handle
 * that's given out by the runtime when
 * the connection gets registered.
 */
struct uart {
    unsigned char handle;

    struct output_pin tx;
    struct input_pin rx;
    struct input_pin* cts;
    struct output_pin* rts;
};

/**
 * Initialize a uart connection. 
 * This registers the connection
 * with the runtime.
 * 
 * @param uart the connection.
 * @return int an error code.
 */
int init_uart(struct uart* uart);

/**
 * Writes a byte about a UART interface.
 * 
 * @param uart the uart connection.
 * @param value the value that should be written.
 * @return int an error code.
 */
int write(const struct uart* uart, unsigned char value);

/**
 * Reads a byte from a UART interface.
 * 
 * @param uart the uart connection.
 * @param value a pointer where the result is stored.
 * @return int an error code.
 */
int read(const struct uart* uart, unsigned char* value);

#endif
