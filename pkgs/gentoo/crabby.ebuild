# Copyright 2025
# Distributed under the terms of the GNU General Public License v3

EAPI=8

DESCRIPTION="Crabby Programming Language"
HOMEPAGE="https://github.com/crabby-lang/crabby"
SRC_URI="https://github.com/crabby-lang/crabby/archive/refs/tags/v${PV}.tar.gz"

LICENSE="GPL3"
SLOT="0"
KEYWORDS="~amd64"
IUSE=""

DEPEND="dev-lang/rust"

RDEPEND="${DEPEND}"

src_compile() {
    cargo build --release || die
}

src_install() {
    dobin target/release/crabby
}
