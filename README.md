<p align="center">
  <img width="210" height="90" src="docs/Hamboo.jpg">
</p>

<img alt="Hamboo" src="docs/watch.jpg"/>

<br>

# Hamboo - Smartwatch based on Esp32-S3 chip.

<img alt="License" src="https://img.shields.io/badge/License-MIT-blue.svg"/>
<img alt="esp-hal" src="https://img.shields.io/badge/Esp_hal-0.17.0-green.svg"/>
<img alt="Slint" src="https://img.shields.io/badge/Slint-1.5.1-green.svg"/>

---


> Main Chip：Esp32-s3 (wifi & bluetooth)
>
> Hardware: Touch screen, microphone, speaker, gyroscope, wireless charging, external RTC, pressure sensor, SDMMC
>
> Software planning: OTA, dial, music player, sports record, games, NFC access bus card, Bluetooth dial, alarm clock, stopwatch, timer...

## Design


### 📦 Blender modeling and 3d printing

![blender.jpg](docs%2Fblender.jpg)

[Hamboo-V4.blend](docs%2FHamboo-V4.blend)

### 🧱 Circuit diagram & PCB

<div>
<img width="35%" height="200" src="docs/schematic.png"/>
<img width="30%" height="200" src="docs/PCB.png"/>
<img width="30%" height="200" src="docs/PCB3D.png"/>
</div>

## 📘 Cost

- **pcb**: ￥0 
- **3d printing**: ￥20
- **bom**: calculating...
- **screen**: ￥30
- **battery**: ￥7
- **watchband**: ￥13
- **others**: ￥30

### ⌨️ Hamboo-rs getting start

```bash
# Environment setup
cargo install espup
espup install
# espup uninstall
export . ~/export-esp.sh
# Firmware 
cargo check
cargo run --release
# Run with simulator
cargo run --features=simulator --release
```

## 🛠️ Planning
- [X] Display
- [X] Touch
- [ ] Other drivers
- [ ] Dial plate
- [ ] Games
- [ ] NFT support