# env-monitor

## Description
**env-monitor** is a set of scripts that utilizes the environmental sensors on
the Raspberry Pi Sense HAT.

`env-monitor` is designed to run under a user named `env-monitor`. Make sure
this user has access to the I2C bus.

Due to a problem of the initialization method of LPS25H in RTIMULib, an one-shot
service `env-monitor-init` is provided to take a measurement and then discard
the result at boot time .

## Author
* [Yishen Miao](https://github.com/mys721tx)

## License
[GNU General Public License, version 2](http://www.gnu.org/licenses/gpl-2.0.html)
