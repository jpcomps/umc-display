### umc-display

quick and dirty oled display app for ssd1306 based oled displays using the UMC and UMC API to display miner data. Connected using J1. In no way production ready and mostly just a proof of concept

### Current Status
Right now just a proof of concept to show that the UMC can be used to drive a display over i2c. Would need alot more polish and error handling for production

The code can be adapted to any micro (ESP32 for example), just need to provide the correct i2c bus and address. 

To run on a local miner, need to change IP_ADDRESS to 127.0.0.1. Current concept was run on a UMC connecting to a remote UMC API endpoint

## Compiling
https://github.com/cross-rs/cross
Use cross to compile for UMC arch: 
```cross build --target armv7-unknown-linux-gnueabihf --release```

```
Usage: umc-display [OPTIONS]

Options:
      --ip <IP>                      [default: 127.0.0.1]
      --i2c-port <I2C_PORT>          [default: /dev/i2c-0]
      --refresh-rate <REFRESH_RATE>  [default: 500]
  -h, --help                         Print help
  -V, --version                      Print version

```
