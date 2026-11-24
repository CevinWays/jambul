pkgname=jambul
pkgver=0.1.0
pkgrel=1
pkgdesc='Interactive notch for KDE Plasma on Wayland'
arch=('x86_64')
url='https://github.com/CevinWays/jambul'
license=('MIT')
depends=('webkit2gtk-4.1' 'gtk3' 'gtk-layer-shell' 'pipewire' 'networkmanager' 'bluez-utils')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
  cd "$pkgname-$pkgver"
  cargo build --release --locked --features tauri/custom-protocol
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm755 target/release/jambul "$pkgdir/usr/bin/jambul"
  install -Dm644 jambul.desktop "$pkgdir/etc/xdg/autostart/jambul.desktop"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
