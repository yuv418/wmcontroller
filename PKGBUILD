# copied from https://aur.archlinux.org/cgit/aur.git/tree/PKGBUILD?h=popsicle
pkgname=wmcontroller
pkgver=0.1.0
pkgrel=0
pkgdesc="Window manager controller, currently just an application launcher"
arch=(x86_64 aarch64 armv7h i686)
license=(Zlib)
# we don't actually want a font to be a dep but for now it is
depends=(fontconfig ttf-jetbrains-mono)
makedepends=(rust cargo)
source=("${pkgname}-${pkgver}.tar.gz::https://github.com/cdknight/wmcontroller/archive/master.tar.gz")

build() {
        cd "${pkgname}-master"
        cargo build --release
}

package() {
        cd "${pkgname}-master"
        mkdir -p $pkgdir/usr/bin/
        cp target/release/wmcontroller $pkgdir/usr/bin/wmcontroller
        install -D LICENSE.org "$pkgdir/usr/share/licenses/$pkgname/LICENSE.org"
}
