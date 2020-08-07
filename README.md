<img src="https://github.com/AWOLASAP/Operating-System/blob/master/OSLogo.png?raw=true" width="256"> 

# Operating-System 

## What's This?
This operating system is a project for the [North West Advanced Programming Workshop](http://nwapw.org/). It is a simple text-based operating system with some graphics as well. Written in Rust, this operating system takes from and builds on various projects that have been done before.

### Core Features
- Boots via QEMU or on real hardware
- VGA text mode and simple 2D graphics
- Keyboard support
- Custom Shell
- Help info for commands
- Tetris!
- Filesystem
- File manipulation
- PC Speaker Audio

### Core UX
The UX is simple at its finest. It includes text output, text input, as well as PC speaker output and some 2D graphics.

## How to run
If you just want to run the operating system on actual hardware, you will need the .bin file which can be found via the releases page. It can then be installed onto a usb drive with something like 'dd' or Balana Etcher, and booted to from a machine. (Currently not working)

However, there is the option of using [QEMU](https://www.qemu.org/), which is the platform we do the majority of developing/testing on. You will need the bootimage-os.bin file as well as the os.tar file in the same directory. After you install QEMU on your system (if your on Windows you will need to add it to your PATH), you can run the following command:

```bash
qemu-system-x86_64 -drive format=raw,file=PATH/TO/bootimage-os.bin -drive if=ide,format=raw,index=1,file=PATH/TO/os.tar -soundhw pcspk
```

If you would like to build this or add on to this project, you first will need [Rust](https://www.rust-lang.org/tools/install). There is also a .bat and .sh file located in the 'os' directory which you can run to install all the necessary rust components. As long as you are in the 'os' directory you can run the following commands:

To build:
```
cargo build --release
```
This will build the rust project and create a bootimage-os.bin file located in `os/target/x86_64-os/release/`

To run:
```
cargo run --release
```
This will build the rust project and automatically run the QEMU command to run the operating system. For this to work you do need QEMU installed and added to your PATH.

### TODO Features
- File editing
- Zork port
- Brainf support
- ACPI implementation

## Tools We Used
- [Rust] (https://www.rust-lang.org/)
- [QEMU] (https://www.qemu.org/)
- Combination of Vim, Atom, and Visual Studio Code
- Trello
- Git/Github
- https://os.phil-opp.com/
- http://osblog.stephenmarz.com/index.html

## People Involved
- [otisdog8](https://github.com/otisdog8)
- [AWOLASAP](https://github.com/AWOLASAP)
- [Alex-x90](https://github.com/Alex-x90)
- [Lolshoc](https://github.com/Lolshoc)

