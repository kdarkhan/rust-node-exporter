pkgname=rust-node-exporter
pkgver=0.2.6
pkgrel=1
pkgdesc='Prometheus metrics exporter for my desktop machine'
arch=('x86_64')
url='https://github.com/kdarkhan/rust-node-exporter'
provides=('rust-node-exporter')
license=('GPL3')
depends=('lm_sensors')
optdepends=('nvidia-utils: fetching nvidia sensors data')
makedepends=(
    'rust'
)
source=()

build() {
    cargo build --release
}

package() {
    install -Dm755 "$startdir/target/release/rust-node-exporter" "$pkgdir/usr/bin/rust-node-exporter"
    install -Dm755 "$startdir/rust-node-exporter.service" "$pkgdir/etc/systemd/system/rust-node-exporter.service"
    install -Dm644 "$startdir/99-kraken.rules" "$pkgdir/etc/udev/rules.d/99-kraken.rules"
}
