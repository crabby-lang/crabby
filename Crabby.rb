# Crabby MacOS support using Homebrew

class Crabby < Formula
    desc "Crabby Programming Language"
    homepage "https://github.com/crabby-lang/crabby"
    url "https://github.com/crabby-lang/crabby/releases/download/v1.2.1/crabby-1.2.1-macos-x86_64.tar.gz"
    sha256 ""
    license "GPL-3.0"

    def install
        bin.install "crabby"
    end

    test do
        system "#{bin}/crabby", "../examples/example.crab"
    end
end
