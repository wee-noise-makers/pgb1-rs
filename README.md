# pgb1-rs

Rust SDK for the [Wee Noise Makers
PGB-1](https://www.crowdsupply.com/wee-noise-makers/wee-noise-makers-pgb-1)
pocket synth/groovebox.

<a href="https://www.crowdsupply.com/wee-noise-makers/wee-noise-makers-pgb-1"><img src="https://www.crowdsupply.com/img/3d7e/2d04eded-e04e-411a-957d-73035af73d7e/pgb1-2-1-top-with-hands-01_jpg_md-xl.jpg" align="center" width="50%" ></a>


# Examples

In this repository you will find two examples of Rust applications that you can
try on the PGB-1.

As of today the examples are configured to run with `probe-rs` and a debug
probe (such as the [Raspberry Pi Debug
Probe](https://www.raspberrypi.com/documentation/microcontrollers/debug-probe.html)).
So make sure the probe is connected to both your computer and the PGB-1.

Trying an example should be as easy as:
 - Go to the example dir (`examples/basics`, `examples/snake`)
 - Build and load with `cargo run`
 
# Quick-start your own project

At this stage the best way to start a new project is to copy one of the
examples and start to modify it.
