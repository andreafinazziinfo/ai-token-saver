class Rtk < Formula
  desc "Rust Context Engine (rtk) — token-saving context engine for AI agents"
  homepage "https://github.com/andreafinazziinfo/rust-context-engine"
  license "Apache-2.0"
  version "2.4.0"

  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/andreafinazziinfo/rust-context-engine/releases/download/v2.4.0/rtk-macos-arm64.tar.gz"
      sha256 "ee5522aae4d335eaedb477275ccadd344d8ddece7f043ba2f66ca8f276041e8c"
    else
      url "https://github.com/andreafinazziinfo/rust-context-engine/releases/download/v2.4.0/rtk-macos-amd64.tar.gz"
      sha256 "fcf6bdf93e11a384ceac4308ccdff34299ceccbc1916ec81a028017850f3cb6b"
    end
  elsif OS.linux?
    url "https://github.com/andreafinazziinfo/rust-context-engine/releases/download/v2.4.0/rtk-linux-amd64.tar.gz"
    sha256 "bb0ca03fc64b58c6e08e8c65ca433e0140eb5e961f9b7d7c9b6a36713568ef52"
  end

  def install
    bin.install "rtk"
  end

  test do
    system "#{bin}/rtk", "--version"
  end
end
