pkgname=disipline-tracker-git
pkgver=0.1.1
pkgrel=1
pkgdesc="Pomodoro and productivity tracker written in Rust with GTK4"
arch=('x86_64')
url="https://github.com/captaincarmnlg/disipline-tracker"
license=('MIT' 'GPL')
depends=('gtk4' 'glib2' 'glibc')
makedepends=('rust' 'cargo' 'pkg-config' 'glib2' 'gtk4')
source=("git+https://github.com/captaincarmnlg/disipline-tracker.git")
sha256sums=('SKIP')
noextract=()

build() {
    cd "$srcdir/disipline-tracker"
      cargo build --release
}

package() {
    cd "$srcdir/disipline-tracker"
    install -Dm755 target/release/disipline-tracker "$pkgdir/usr/bin/disipline-tracker"
    install -Dm644 data/disipline-tracker.desktop "$pkgdir/usr/share/applications/disipline-tracker.desktop"
    install -Dm644 data/icon.png "$pkgdir/usr/share/icons/hicolor/128x128/apps/disipline-tracker.png"
    install -Dm644 data/sounds/* "$pkgdir/usr/share/disipline-tracker/sounds/"
}
