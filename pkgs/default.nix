{ stdenv, fetchurl }

stdenv.mkDerivation rec {
    pname = "crabby";
    version = "1.2.1";

    src = fetchurl {
        url = "https://github.com/crabby-lang/crabby/archive/refs/tags/v${version}.tar.gz";
        sha256 = "1i0dgy48ds7bwbmwhshijn2syncpqxiv8wg2yh7hrys1537a9ryy";
    };

    unpackPhase = "tar xzf $src";
    installPhase = ''
        mkdir -p $out/bin
        cp crabby $out/bin/crabby
    '';

    meta = with stdenv.lib; {
        description = "Crabby Programming Language";
        homepage = "https://github.com/crabby-lang/crabby";
        license = license.gpl3;
        platforms = platforms.linux;
        maintainers = [ maintainers.Kazooki123 ];
    };
}
