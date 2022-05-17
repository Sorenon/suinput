## Workspace Members

### src/
TEMPORARY  
Common types and functions  
TODO move files into other members and delete package

### Bevy Testing
TEMPORARY  
Bevy plugin development

### Drivers
PUBLIC  
Contains the default input drivers

### Generator
INTERNAL  
Code generator

### Loader
PUBLIC  
Loads the embedded runtime

TODO:
Load the external runtime or the embedded runtime if none are found

## Loader C-API
PUBLIC  
TODO  
Provides an FFI interface for the loader

### Runtime API
PUBLIC  
The runtime's rust API  
Provides an abstraction over the embedded runtime or an external runtime

### Runtime Impl
INTERNAL  
The actual runtime logic

### Runtime C-API
PUBLIC  
TODO  
Provides an FFI interface for the runtime

### Winit Testing
TEMPORARY  
General development testing
