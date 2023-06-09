### Supports only 1.66A exe currently.

### Features
- Adjust global gravity
- Adjust normal mode additional gravity when a wheel is lifted off the ground
- Force HC mode physics on player and racing bots in normal mode

By default, when a wheel is lifted off the ground in normal (non hardcore) mode, an extra gravity modifier of 1.0 is added to the vehicle to quickly put the vehicle back onto the ground. The value is statically saved in the exe, and this patcher changes that value by changing an instruction.

Additionally gravity used by havok and other bits of the game can also be changed from the default of -9.81, which might be useful for hardcore mode.

0.0 gravity modifier adds no additional gravity, negative values such as -10.0 that overcomes default gravity will send the car upward at curbs and jumps.

0.0 gravity modifier demo: https://streamable.com/ldur6j

Ghidra and x64dbg were used to seek out the instruction for patching as well as assembling the new instruction, patching methodology can be found at https://github.com/Kethen/TDU_normal_mode_gravity_changer/blob/main/src/util.rs

iced-rs is used for the user interface, checksums is used for identifying exe files, rfd is used for providing a file picker.

.exe for windows (win32 and win64), without for linux (x86_64 glibc).

Alternatively the project can be built using cargo like other rust projects, it should build in MacOS as well.
