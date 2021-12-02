#ifndef _GPIO_H_
#define _GPIO_H_

#include "runtime.h"

/**
 * Structure for an input pin.
 * Holds information about the
 * port and pin number.
 * In order to register it with
 * the hardware you need to call
 * `gpio_input_init`.
 */
struct input_pin {
    unsigned int port;
    unsigned int pin;
};

/**
 * Structure for an output pin.
 * Holds information about the
 * port and pin number.
 * In order to register it with
 * the hardware you need to call
 * `gpio_output_init`.
 */
struct output_pin {
    unsigned int port;
    unsigned int pin;
};

/**
 * Initialize the given input pin with the
 * hardware.
 * @param ip the input pin. 
 * @return int an error code.
 */
int gpio_input_init(struct input_pin* ip);

/**
 * Initialize the given output pin with the
 * hardware.
 * @param ip the output pin. 
 * @return int an error code.
 */
int gpio_output_init(struct output_pin* op);

/**
 * Set a given output pin to high.
 * @param op the output pin.
 * @return int an error code.
 */
int set_high(struct output_pin* op);

/**
 * Set a given output pin to low.
 * @param op the output pin.
 * @return int an error code.
 */
int set_low(struct output_pin* op);

/**
 * Check if a given input pins value is low.
 * 
 * @param ip the input pin.
 * @param location_of a bool pointer where the 
 * result will be written to.
 * @return int an error code.
 */
int is_low(const struct input_pin* ip, bool* location_of);

/**
 * Check if a given input pins value is high.
 * 
 * @param ip the input pin.
 * @param location_of a bool pointer where the 
 * result will be written to.
 * @return int an error code.
 */
int is_high(const struct input_pin* ip, bool* location_of);

#endif
