# rust-node-exporter

[Prometheus](https://prometheus.io/) metrics exporter to expose custom metrics from
my desktop. I am using this project to learn Rust, Prometheus, and Grafana.
Currently it exposes:
* NZXT Kraken X52 metrics
* `lm_sensors` output with `asus-wmi-sensors` driver
* SSD temps using `hddtemp` daemon
* Nvidia metrics using `nvidia-smi -q`

![Prometheus UI screenshot](prometheus-screenshot.png?raw=true)
