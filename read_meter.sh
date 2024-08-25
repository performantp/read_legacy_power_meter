#!/bin/bash

# Define the serial port
SERIAL_PORT="/dev/ttyUSB0"

# Configure the serial port
stty -F $SERIAL_PORT 300 cs7 parenb -parodd -cstopb

# Send the initial request command to the meter
echo -ne "/?!\r\n" > $SERIAL_PORT

# Read and display the response from the meter
cat $SERIAL_PORT
