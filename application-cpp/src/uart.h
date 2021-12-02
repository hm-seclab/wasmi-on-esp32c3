#pragma once
#include "gpio.h"
#include "runtime.h"

/**
 * A class that holds information about the pins
 * relevant for UART.
 */
class Pins {
   public:
    InputPin rx;
    OutputPin tx;
    InputPin* cts;
    OutputPin* rts;

    /**
     * Construct a new Pins object.
     *
     * @param rx the receiver pin.
     * @param tx the sender pin.
     * @param cts enable receiver pin (optional through nullpointer).
     * @param rts enable sender pin (optional through nullpointer).
     */
    Pins(InputPin rx, OutputPin tx, InputPin* cts, OutputPin* rts)
        : rx(rx), tx(tx), cts(cts), rts(rts) {}
};

/**
 * A class that handles an active UART connection.
 * The connection is identified by the related pins
 * and the handle that's obtained when the connection
 * is registered.
 */
class Uart {
   private:
    Pins pins;
    bool is_initialized;
    unsigned char handle;

    int init();

   public:
    /**
     * Construct a new Uart object.
     *
     * @param pins the pins used for the connection.
     */
    Uart(Pins pins) : pins(pins), is_initialized(false), handle(0) {}

    ~Uart() = default;

    /**
     * Writes a single byte over the UART interface.
     *
     * @param word the byte.
     * @return int an error code.
     */
    int write(unsigned char word);

    /**
     * Reads a single byte from the UART interface.
     *
     * @param value the byte.
     * @return int the value.
     */
    int read(unsigned char* value);
};
