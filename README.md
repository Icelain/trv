# trv

A simple web-based video/audio transcriber in rust. Proud ffmpeg and openai-whisper wrapper.

# Installation and Configuration
```git clone https://github.com/icelain/trv```

In cargo.toml, set the whisper.cpp dependency's feature to whatever is available on your system. Eg: cuda/metal/rocm.

Then,
```cargo run release```
