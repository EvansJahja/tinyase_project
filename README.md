# (WIP) Tinyase: A no-std and mostly zerocopy Aseprite (.ase/.aseprite) file parser for embedded devices

## What is it
There are other .aseprite reader out there but none of them are no-std safe and/or zerocopy focused. I want to use aseprite file on an embedded device with pretty much no memory to spare, so I made this library.

![mouse.png](img/mouse.png)

## Status
Work in progress, expect breaking changes, but currently the library works for basic indexed color image. I'm still hardcoding a lot of stuff for now.
