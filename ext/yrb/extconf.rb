# frozen_string_literal: true

require "mkmf"
require "rb_sys/mkmf"

create_rust_makefile("yrb") do |r|
  # A source install builds the crate graph Cargo.lock pins, or fails.
  r.extra_cargo_args = ["--locked"]

  # rb-sys always exports RUSTFLAGS, so .cargo/config.toml is skipped; the
  # macOS 27 linker writes an unloadable bundle when it strips dead code.
  if RbConfig::CONFIG["host_os"].include?("darwin")
    r.extra_rustflags = ["-C", "link-dead-code=on"]
  end
end
