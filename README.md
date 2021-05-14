# rust-node-exporter

[Prometheus](https://prometheus.io/) metrics exporter to expose custom metrics from
my desktop. I am using this project to learn Rust and Prometheus.
Currently it exposes:
* NZXT Kraken X52 metrics which it gets from
  [liquidctl](https://github.com/liquidctl/liquidctl).
* `lm_sensors` output with `asus-wmi-sensors` driver.

![Prometheus UI screenshot](prometheus-screenshot.png?raw=true)
