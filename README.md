# jack-autoplug

A tiny utility that ensures that a certain set of JACK ports are always connected to each other (if they are present).

## Building

Like any Rust application: `cargo build --release` produces the binary at `target/release/jack-autoplug`.

## Running

Check out `--help` for a synopsis. For a practical example, I use this with PipeWire's JACK API emulation to take the
audio from a HDMI capture card and patch it through to my headphones immediately like this (names replaced with dummy
values for clarity):

```bash
$ pw-jack jack-autoplug -f "HDMI Capture Card Analog Stereo" -F capture_FL -F capture_FR -t "Headphone Device" -T playback_FL -T playback_FR
```

This connects the `capture_FL` port of the capture card with the `playback_FL` port of the headphones, and same for the
FR ports. If any of the two devices vanishes, the port connections will obviously be lost, but jack-autoplug will
restore them as soon as the device reappears.
