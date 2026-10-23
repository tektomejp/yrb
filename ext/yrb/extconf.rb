# frozen_string_literal: true

require "mkmf"
require "rb_sys/mkmf"

create_rust_makefile("yrb") do |r|
  # rb-sys always exports RUSTFLAGS, so .cargo/config.toml is skipped; the
  # macOS 27 linker writes an unloadable bundle when it strips dead code.
  if RbConfig::CONFIG["host_os"].include?("darwin")
    r.extra_rustflags = ["-C", "link-dead-code=on"]
  end
end
