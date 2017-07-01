#!/usr/bin/python
"""
sense.py: Monitoring environment with Sense HAT.
"""
import time
import argparse
from sense_hat import SenseHat

def main():
    """
    main function
    """

    sense = SenseHat()

    parser = argparse.ArgumentParser(description="write sensor value to file")

    parser.add_argument(
        "--init",
        action="store_true",
        help="initialize sensors. Data are discarded."
    )

    arguments = parser.parse_args()

    result = (
        time.time(),
        sense.get_pressure(),
        sense.get_temperature_from_pressure(),
        sense.get_humidity(),
        sense.get_temperature_from_humidity()
    )

    if arguments.init:
        pass
    else:
        with open("records.tsv", "a", newline="\n") as data:
           data.write("{}\t{}\t{}\t{}\t{}\n".format(*result)) 

if __name__ == "__main__":
    main()
