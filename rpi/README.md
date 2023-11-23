## Build

Usefull stuff:
`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
`cargo install cross --git https://github.com/cross-rs/cross`


Build for PI:
> make sure docker is installed

`cross build --target arm-unknown-linux-gnueabihf --release`
