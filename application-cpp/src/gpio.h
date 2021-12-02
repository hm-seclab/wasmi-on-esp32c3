#pragma once
#include "runtime.h"

/**
 * A Gpio Pin.
 * Different methods are available based on the spezialisation of the type.
 */
class Pin {
   protected:
    /**
     * Port and pin of the GPIO.
     */
    unsigned int port;
    unsigned int pin;
    bool is_initialized;

    /**
     * Initialize a Pin on the underlying hardware.
     *
     * @param is_input whether the pin is an input pin or not.
     * @return int error code.
     */
    int init(bool is_input);

   public:
    /**
     * Construct a new Pin object.
     *
     * @param port the port of the GPIO.
     * @param pin the pin of the GPIO.
     */
    Pin(unsigned int port, unsigned int pin) {
        this->port = port;
        this->pin = pin;
        this->is_initialized = false;
    }

    /**
     * Destroy the Pin object.
     * Frees underlying hardware ressources.
     */
    ~Pin() { gpio_deinit(port, pin); }

    /**
     * Get the port of the GPIO.
     *
     * @return int the port.
     */
    unsigned int get_port() { return port; }

    /**
     * Get the pin of the GPIO.
     *
     * @return int the pin.
     */
    unsigned int get_pin() { return pin; }
};

class InputPin : public Pin {
   public:
    InputPin(unsigned int port, unsigned int pin) : Pin(port, pin) {}

    /**
     * For input pin: check if the pin is high.
     *
     * @param result location of the result.
     * @return int error code.
     */
    int is_high(bool* result);

    /**
     * For input pin: check if the pin is low.
     *
     * @param result location of the result.
     * @return int error code.
     */
    int is_low(bool* result);
};

class OutputPin : public Pin {
   public:
    OutputPin(unsigned int port, unsigned int pin) : Pin(port, pin) {}
    /**
     * For output pin: set the pin to high.
     *
     * @return int error code.
     */
    int set_high();

    /**
     * For output pin: set the pin to low.
     *
     * @return int error code.
     */
    int set_low();
};
