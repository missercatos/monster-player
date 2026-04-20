# Maintainer: Your Name <youremail@domain.com>
pkgname=monster-player
pkgver=0.2.4
pkgrel=1
pkgdesc="TUI music player with local playback and Siren Records streaming"
arch=('x86_64' 'aarch64')
url="https://github.com/yourusername/monster-player"
license=('MIT')
depends=('alsa-lib' 'dbus' 'libchromaprint' 'cava')
makedepends=('rust' 'cargo' 'pkg-config')
provides=('tmplayer')
conflicts=('tmplayer')
source=("$pkgname-$pkgver.tar.gz::https://github.com/yourusername/monster-player/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

prepare() {
  cd "$pkgname-$pkgver"
  cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
}

build() {
  cd "$pkgname-$pkgver"
  export RUSTUP_TOOLCHAIN=stable
  export CARGO_TARGET_DIR=target
  cargo build --frozen --release --all-features
}

check() {
  cd "$pkgname-$pkgver"
  cargo test --frozen --all-features
}

package() {
  cd "$pkgname-$pkgver"
  # Try new binary name first, fallback to old name
  if [[ -f "target/release/monsterplayer" ]]; then
    install -Dm755 "target/release/monsterplayer" "$pkgdir/usr/bin/monsterplayer"
  elif [[ -f "target/release/tmplayer" ]]; then
    install -Dm755 "target/release/tmplayer" "$pkgdir/usr/bin/monsterplayer"
  else
    echo "ERROR: Binary not found at target/release/{monsterplayer,tmplayer}"
    exit 1
  fi
  ln -s monsterplayer "$pkgdir/usr/bin/tmplayer"
  
  # desktop entry
  install -Dm644 /dev/null "$pkgdir/usr/share/applications/monsterplayer.desktop"
  cat > "$pkgdir/usr/share/applications/monsterplayer.desktop" <<EOF
[Desktop Entry]
Name=Monster-Player
Comment=TUI Music Player with Siren Records Streaming
Exec=monsterplayer
Icon=utilities-terminal
Terminal=true
Type=Application
Categories=AudioVideo;Audio;Player;
Keywords=music;player;tui;terminal;streaming;
EOF

  # license
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
  
  # documentation
  install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
}