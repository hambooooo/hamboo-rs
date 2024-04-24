<p align="center">
  <img width="210" height="90" src="docs/Hamboo.jpg">
</p>
<p align="center">
    <img alt="License" src="https://img.shields.io/badge/license-MIT-blue.svg"/>
    <img alt="esp-hal" src="https://img.shields.io/badge/esp_hal-0.17.0-green.svg"/>
    <img alt="Slint" src="https://img.shields.io/badge/slint-1.5.1-green.svg"/>
</p>

<img alt="Hamboo" src="docs/watch.jpg"/>

<br>

# Hamboo - ⌚Smartwatch based on Esp32-S3 chip.

---

> **Main Chip**：Esp32-s3 (wifi & bluetooth) <br>
> **Hardware**: Touch screen, microphone, speaker, gyroscope, wireless charging, external RTC, pressure sensor, SDMMC <br>
> **Software Planning**: OTA, dial, music player, sports record, games, NFC access bus card, Bluetooth dial, alarm clock, stopwatch, timer... <br>

## Design

### 📐 Blender modeling and 3d printing

![blender.jpg](docs%2Fblender.jpg)

[Hamboo-V4.blend](docs%2FHamboo-V4.blend)

### 🖥️ Circuit diagram & PCB

<div>
<img width="30%" height="200" src="docs/circuit-diagram.png"/>
<img width="30%" height="200" src="docs/PCB.png"/>
<img width="30%" height="200" src="docs/PCB3D.png"/>
</div>

[hamboo-pcb.zip](docs%2Fhamboo-pcb.zip)

## 💰 Cost & 🎧Peripheral

- **pcb**: ￥0 （4-layer board free play [lceda](https://lceda.cn/)）
- **3d printing**: ￥20 （White resin material）
- **bom**: ¥? (calculating...)
- **screen**: ￥30 （P169H002-CTP 1.69inch）
- **battery**: ￥7 (size: 302530, 3.7v 250mAh)
- **watchband**: ￥13 (for apple watch(7/8/9))
- **others**: ￥30 (speaker size: 1506、motor size: 3610、wireless charging coil: 3021 12.5uH)

<div>
    <img height="100" src="docs/screen.jpg"/>
    <img height="100" src="docs/motor.jpg"/>
    <img height="100" src="docs/speaker.jpg"/>
    <img height="100" src="docs/coil.png"/>
    <img height="100" src="docs/battery.jpg"/>
    <img height="100" src="docs/watchband.jpg"/>
    <img height="100" src="docs/3dmodel.jpg"/>
</div>

## ⌨️ hamboo-rs

### 🎬 Getting Start

```bash
# Setting environment
cargo install espup
espup install
# To uninstall
# espup uninstall
export . ~/export-esp.sh
# Firmware 
cargo check
cargo run --release
# Run with simulator
cargo run --features=simulator --release
```

### 📋 Planning

- [X] Display
- [X] Touch
- [ ] Slint
- [ ] Other drivers
- [ ] OTA
- [ ] Watch dial
- [ ] 🎮 Games
- [ ] NFT support

### 📄 License

[MIT](https://opensource.org/licenses/MIT)

Copyright (c) 2014-present, Michael