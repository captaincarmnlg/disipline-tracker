pkgname=disipline-tracker-git
pkgver=0.1.0
pkgrel=1
pkgdesc="Pomodoro and productivity tracker written in Rust with GTK4"
arch=('x86_64')
url="https://github.com/captaincarmnlg/disipline-tracker.git"
license=('MIT' 'GPL')
depends=('gtk4' 'glib2' 'glibc')
makedepends=('rust' 'cargo' 'pkg-config' 'glib2' 'gtk4')
source=()
noextract=()

build() {
    cd "$srcdir"
    cargo install --path . --root "$pkgdir/usr" --locked
}

package() {
    cd "$srcdir"

    # Optional: install a .desktop file
    install -Dm644 data/disipline-tracker.desktop "$pkgdir/usr/share/applications/disipline-tracker.desktop"
    
    # Optional: install an icon if you have one
    install -Dm644 data/icon.png "$pkgdir/usr/share/icons/hicolor/128x128/apps/disipline-tracker.png"
}
