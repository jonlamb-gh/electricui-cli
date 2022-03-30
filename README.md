# electricui-cli &emsp; ![ci] [![crates.io]](https://crates.io/crates/electricui-cli)

An unofficial and incomplete CLI for devices implementing the [ElectricUI Binary Protocol][eui-bin-proto].

See the [ElectricUI docs][eui-docs] or the [ElectricUI Rust library][eui-rust-lib] for more information.

## Examples

See [electricui-embedded-stm32f4-example crate][eui-stm32-example] for an example target.

### Basic checks

```
electricui check /dev/ttyUSB0

Board ID: 0xBEEF
Board name: my-board
Message IDs (4):
  led_blink
  led_state
  lit_time
  name
IDs count: 4
Variables:
  Id(led_blink), Kind(U8(1))
  Id(led_state), Kind(U8(1))
  Id(lit_time), Kind(U16(200))
  Id(name), Kind(CharArray(my-board))
Heartbeat: 5, matches: true
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

[ci]: https://github.com/jonlamb-gh/electricui-cli/workflows/CI/badge.svg
[crates.io]: https://img.shields.io/crates/v/electricui-cli.svg
[eui-docs]: https://electricui.com/docs/
[eui-bin-proto]: https://electricui.com/docs/hardware/protocol
[eui-rust-lib]: https://github.com/jonlamb-gh/electricui-embedded-rs
[eui-stm32-example]: https://github.com/jonlamb-gh/electricui-embedded-stm32f4-example
