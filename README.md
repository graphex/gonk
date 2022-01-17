I got a Zenith Space Command remote control off E-Bay, and wanted to make something to listen to it.

Full Disclosure: This is my first Rust microcontroller, full no_std project. The goal is primarily to learn the concepts of microcontroller programming and the caveats of rust no_std. The layout of this program is atrocious. Instead of a `for` loop to collect the sample to pass to the FFT algorithm, it should configure the DMA controller to buffer peripheral-clock driven ADC readings to a buffer which triggers an interrupt when full, and starts the FFT from the interrupt. I'm not there, though, with either my Rust knowledge or my STM32 Datasheet/Reference Manual deciphering abilities. I also think this would do well with RTIC driving the architecture... I'm not there, either. If someone reading this is able and interested in writing a reference implementation that does things "right" I'll send you a Daisy to help you get started.

I connected an Elecro-Smith Daisy (which has a STM32H750IBKX processor) to a breadboard, then added the following:
 * an Adafruit 8x8 bicolor LED matrix with I2C backpack http://adafru.it/901 to pins #12 and #13 (through the logic level converter)
 * a digital logic level converter to go from the 3v3 of the Daisy to the 5v recommended for the I2C matrix
 * a SparkFun Analog MEMS Microphone BOB-18011 with the audio output connected to pin #22 (ADC0)
 * a Keysight InfiniiVision MSOX3024T oscilloscope connected to pin #14 (SEED_PIN_13) so I could see what sort of sample rates I was getting, and so I could figure out what the heck was going on in general

In building this project, I found that both I2S and the Daisy's built-in audio codec would not really help me out in reading the ultrasonic signals form the Zentih Space Command remote. The remote has no actual circuitry, just strikers that basically ring bells (aluminium rods) which vibrate at frequencies between 37.8kHz and 41.4kHz. While my dog can arguably hear these frequencies, humans can definitely not. Because of this, most things optimized for "audio" usage will shed these ultrasonic frequencies like the dickens. While this realization made completing this project much more difficult, I also learned a lot more as a result.

Watching the oscilloscope showed me that the sample rate was a pretty solid 450kHz sample rate, which gave me a pretty solid (ok slightly drift-y) target for the FFT sample rate that was more than 10x the frequency I was hunting for. Once a sample is taken, an FFT finds a spectrum of powers and they are converted to ranges corresponding to the Zenith Space Command remote control's frequencies:
 * Channel Down: 40.38kHz
 * Volume: 37.38kHz
 * Off/On: 38.88kHz
 * Channel Up: 41.38kHz

Things I hope to do:
 * Also look in the 697Hz - 1633Hz ranges and do DTMF (Touch Tone®) detection (AKA reverse phreaking)
 * Either setup DMA with a stable clock, or evaluate the realtime clock to account for sample rate drift
 * hook up the I2C bus to a few 14 segment displays (http://adafru.it/1910 or 1908) and show the detected button(s) pressed.
 * send out USB/I2C/... events when a button is pressed so that a Raspberry Pi or Electric Imp can take some sort of action like IFTTT or LAN TCP/HTTPS API integration
 * try out Goertzel filters instead of FFTs, potentially making continuous monitoring possible (current FFT approach takes longer than realtime to evaluate samples) -- right now all Goertzel implementations aren't usable in no_std without modification
 * re-frame this in RTIC to better manage the microcontroller resources
 * abstract this to work with a wider variety of microcontrollers (as long as they can do ADC sampling >400kHz)
 * outbound event signaling with Ockam libraries to provide better security
 * separate project to output DTMF signals from a keypad, and maybe to send the ultrasonic frequencies that the Zenith Space Command remote sends



