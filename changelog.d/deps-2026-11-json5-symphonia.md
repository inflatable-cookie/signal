Upgrade json5 to 1.x and symphonia to 0.6; media decoding still keeps encoder delay and padding untrimmed.
Ogg Vorbis decodes now start with a 1024-frame lead-in that 0.5 dropped, and AAC output differs by less than one 24-bit LSB.
