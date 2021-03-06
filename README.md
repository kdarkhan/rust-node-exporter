# rust-node-exporter

[Prometheus](https://prometheus.io/) metrics exporter to expose custom metrics from
my desktop. I am using this project to learn Rust, Prometheus, and Grafana.
Currently it exposes:
* NZXT Kraken X52 metrics
* `lm_sensors` output with `asus_wmi` module, using Rust FFI call to `libsensors`
* SSD temps using `hddtemp` daemon
* Nvidia metrics using `nvidia-smi -q`
* `/proc/meminfo`, `/proc/cpuinfo`, and `/proc/net/dev` metrics

![Prometheus UI screenshot](prometheus-screenshot.png?raw=true)
