# basicvideostreamer
![License: MIT](https://img.shields.io/github/license/jhnw/basicvideostreamer)
[![Tests](https://github.com/JhnW/basicvideostreamer/actions/workflows/tests.yml/badge.svg?branch=main)](https://github.com/JhnW/basicvideostreamer/actions/workflows/tests.yml)
![GitHub issues](https://img.shields.io/github/issues/jhnw/basicvideostreamer)
[![crates.io](https://img.shields.io/crates/v/basicvideostreamer.svg)](https://crates.io/crates/basicvideostreamer)
[![Released API docs](https://docs.rs/basicvideostreamer/badge.svg)](https://docs.rs/httparse)
![Crates.io - Download](https://img.shields.io/crates/d/basicvideostreamer)

Simple Rust video streaming library using HTTP 1.1 model.

The current version currently allows only jpg images to be sent using multipart content type from HTTP.
The implementation uses internal threads without blocking the main program. 
It is possible a primitive library, but I think quite useful.

If you have any suggestions, comments, requests - don't hesitate to subscribe to me directly (check repository owner) or leave me a ticket on GitHub.
Pull request are also welcome. I can make a simple feature request quickly (such as enabling png streaming) - I just need to know if there is a demand.
