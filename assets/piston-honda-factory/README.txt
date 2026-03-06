PISTON HONDA MARK III - FACTORY WAVESET INSTRUCTIONS

Equipment needed: MicroSDcard.


Steps:

1. Format the microSD card using your computer to “FAT” format.
2. Copy all eight .WAV files in this collection to the root level of the SD card. Do not rename them.
3. Eject the SD card and remove it from your computer.
4. Insert the SD card into the front panel slot of the Piston Honda Mark III. 
5. Hold down the Piston's black rotary encoder button and press the "LINK" button to enter the OPTIONS menu. Release both buttons.
6. Turn the rotary encoder to scroll down to "Load Waves From SD". It's on the second page of options.
7. Press the rotary encoder button. The Piston Honda will check for the presence of 8 wave files named "1.wav" ~ "8.wav". If these files are not present, the operation will fail. If the files are valid, the Piston will load each of them into memory and then restart.

You should keep a backup of your wave configurations on the SD card or your computer. If you upgrade the firmware on the Piston Honda, the onboard waveforms will be reset to this factory data, so you must reload your custom waveforms.


FILE FORMAT / CREATING YOUR OWN

Any mono 16 bit WAV file will work with Piston Honda, but it likely won't work as expected unless you've formatted the wave data correctly. Use the editors mentioned at the end of this document to create and arrange the wave banks, or generate your own through other means!  Each WAV file should consist of 64 single-cycle waves measuring 256 samples each. The WaveEdit program linked below correctly generates files in this format if all 64 wave slots in that program are filled. 

Each of the eight WAV files to be loaded into the Piston Honda represents one of the 8 possible positions on the Z axis slider. A group of 8 waves in sequence within the file corresponds to the 8 values of the X slider, and the collection of these 8 groups within the file correspond to the values selected by the Y slider.

IMPORTANT -  When you put your waves on the SD card, you must load the entire memory of 8 WAV files at once. They MUST be named "1.wav", "2.wav" all the way up to "8.wav". The files will not load if they are not named correctly, or if any of the 8 are missing. It is OK to have other files with different names stored on the card, but only the WAV files with correct file names will be loaded into the module.


FREE EDITORS:

http://synthtech.com/waveedit/ - by Synthesis Technology
http://scw.sheetsofsound.com/ - By Darwin Grosse

FACTORY WAVESET INFO:

Z1: Classic Waveforms 1 - (traditional)
Z2: Lucifer Additive - Scott Jaeger
Z3: Classic Waveforms 2 - (traditional)
Z4: LFSR Vectors - Scott Jaeger
Z5: Radek Rudnicki
Z6: Joey Blush
Z7: Rodent
Z8: Surachai Sutthisasanakul
