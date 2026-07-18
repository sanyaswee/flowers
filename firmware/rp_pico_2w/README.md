# Raspberry Pi Pico 2W based node
This is the driver for the RP Pico 2W node version. **Up to 2 flowers**

## Node Capabilities
The main limitation of this node is that there are only 3 ADC peripherals onboard. Giving up one for the water level leaves us just 2 for the flowers

| Telemetry       | Hardware                                                                                                 | Interface     |
|-----------------|----------------------------------------------------------------------------------------------------------|---------------|
| Water tank      | Water level sensor (exact level)                                                                         | Analog        |
| Temperature     | [BMP280](https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bmp280-ds001.pdf) | Digital / I2C |
| Pressure        | [BMP280](https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bmp280-ds001.pdf) | Digital / I2C | 
| Light Intensity | [BH1750FVI](https://www.alldatasheet.com/datasheet-pdf/view/338083/ROHM/BH1750FVI.html)                  | Digital / I2C |

## Schematics
![Schematic](schematic.svg)