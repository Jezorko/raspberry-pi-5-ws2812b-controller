# Timings

According to [josh.com](https://wp.josh.com/2014/05/13/ws2812-neopixels-are-not-so-finicky-once-you-get-to-know-them/),
the necessary precise timings for driving NeoPixels are:

| **Symbol** |       **Parameter**       | **Min** | **Typical** | **Max** | **Units** |
|:----------:|:-------------------------:|:-------:|:-----------:|:-------:|:---------:|
|     T0H    | 0 code, high voltage time |   200   |     350     |   500   |     ns    |
|     T1H    | 1 code, high voltage time |   550   |     700     |  5 500  |     ns    |
|     TLD    |   data, low voltage time  |   450   |     600     |  5 000  |     ns    |
|     TLL    |  latch, low voltage time  |  6 000  |             |         |     ns    |


T0H:
    350 / 125 == 2.8
    3 * 125 = 375
    3 bits => 111
T0L:
    600 / 125 == 4.8
    5 * 125 = 625
    5 bits => 00000
T0 signal -> 11100000

T1H:
    700 / 125 == 5.6
    6 * 125 = 750
    6 bits => 111111
T1L:
    600 / 125 == 5
    5 * 125 = 625
    5 bits => 00000
T1 signal -> 11111100000

Adjusted to 2 bytes

T0 -> 11100000 00000000
T1 -> 11111100 00000000
latch -> 251 zeroed-out bytes