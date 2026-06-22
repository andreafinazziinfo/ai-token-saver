class Rtk < Formula
  desc "Rust Context Engine (rtk) — token-saving context engine for AI agents"
  homepage "https://github.com/andreafinazziinfo/rust-context-engine"
  license "MIT"

  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/andreafinazziinfo/rust-context-engine/releases/latest/download/rtk-macos-arm64.tar.gz"
      # sha256 "PLACEHOLDER_SHA256_MAC_ARM64"
    else
      url "https://github.com/andreafinazziinfo/rust-context-engine/releases/latest/download/rtk-macos-amd64.tar.gz"
      # sha256 "PLACEHOLDER_SHA256_MAC_AMD64"
    end
  elsif OS.linux?
    url "https://github.com/andreafinazziinfo/rust-context-engine/releases/latest/download/rtk-linux-amd64.tar.gz"
    # sha256 "PLACEHOLDER_SHA256_LINUX_AMD64"
  end

  def install
    bin.install "rtk"
  end

  test do
    system "#{bin}/rtk", "--version"
  end
end
