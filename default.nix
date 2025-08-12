{ stdenv, fetchurl }

stdenv.mkDerivation rec {
    pname = "crabby";
    version = "1.2.1";

    src = fetchurl {
        url = "https://github.com/crabby-lang/crabby/release/download/${version}/crabby-${version}-x86_64-linux.tar.gz";
        sha256 = "";
    };

    unpackPhase = "tar xzf $src";
    installPhase = ''
        mkdir -p $out/bin
        cp crabby $out/bin/crabby
    '';

    meta = with stdenv.lib; {
        description = "Crabby programming language compiler & interpreter";
        homepage = "https://github.com/crabby-lang/crabby";
        license = license.gpl3;
        platforms = platforms.linux;
        maintainers = [ maintainers.Kazooki123 ];
    };
}
