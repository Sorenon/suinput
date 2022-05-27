# SuInput Development
Design docs: [Action System Design](https://sorenon.github.io/Action-System/)

## Repo Overview
### suinput
The runtime's rust API  
Provides an abstraction over the embedded runtime or an external runtime

### drivers/*
The default input drivers  
TODO Examine other HID driver APIs (e.g. Monado)

### generator 
Internal code generation

### suinput-core
Internal runtime implementation

### suinput-types
Shared rust types

### suinput-ffi (TODO)
The C API

### Winit Testing
General development testing

### Bevy Testing
Bevy plugin development (temporarily inactive)
