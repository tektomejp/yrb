# frozen_string_literal: true

require "bundler"
require "open3"
require "tmpdir"

repository = File.expand_path("..", __dir__)
revision, status = Open3.capture2("git", "-C", repository, "rev-parse", "--verify", "#{ARGV.fetch(0, 'HEAD')}^{commit}")
abort "Cannot resolve the requested revision" unless status.success?
revision = revision.strip

smoke = <<~'RUBY'
  require "y-rb"

  extension = $LOADED_FEATURES.find { |path| path.end_with?("/yrb.bundle", "/yrb.so") }
  raise "Loaded an extension outside the fresh installation" unless extension && File.realpath(extension).start_with?(File.realpath(ENV.fetch("BUNDLE_PATH")) + "/")

  [[:diff, :sync], [:diff_v2, :sync_v2]].each do |encode, apply|
    local = Y::Doc.new
    local.get_text("text") << "Hello 日本語 🌍"
    remote = Y::Doc.new
    remote.public_send(apply, local.public_send(encode))
    raise "Unicode synchronization failed" unless remote.get_text("text").to_s == "Hello 日本語 🌍"

    remote.get_text("text") << "!"
    local.public_send(apply, remote.public_send(encode))
    raise "Return synchronization failed" unless local.get_text("text").to_s == "Hello 日本語 🌍!"

    local.get_text("text").slice!(0, 6)
    restored = Y::Doc.new
    restored.public_send(apply, local.public_send(encode))
    raise "Deleted text reappeared" unless restored.get_text("text").to_s == "日本語 🌍!"
  end

  raise "The broken build dependency remained locked" unless Gem.loaded_specs.fetch("rb_sys").version >= Gem::Version.new("0.9.130")
  puts "FRESH_INSTALL_OK: #{extension} (rb_sys #{Gem.loaded_specs.fetch('rb_sys').version})"
RUBY

Bundler.with_unbundled_env do
  Dir.mktmpdir("yrb-fresh-install-") do |directory|
    environment = ENV.keys.grep(/\ABUNDLE_/).to_h { |key| [key, nil] }.merge(
      "BUNDLE_GEMFILE" => File.join(directory, "Gemfile"),
      "BUNDLE_PATH" => File.join(directory, "bundle"),
      "BUNDLE_IGNORE_CONFIG" => "1",
      "BUNDLE_DISABLE_SHARED_GEMS" => "true",
      "CARGO_TARGET_DIR" => File.join(directory, "target"),
      "RUBYOPT" => nil,
      "RUBYLIB" => nil
    )
    gemfile = environment.fetch("BUNDLE_GEMFILE")
    File.write(gemfile, "source 'https://rubygems.org'\ngem 'rb_sys', '= 0.9.128'\n")
    puts "Seeding the old rb_sys lock in #{directory}", "Testing yrb revision #{revision}"
    system(environment, Gem.ruby, "-S", "bundle", "lock", chdir: directory, exception: true)

    File.write(gemfile, <<~GEMFILE)
      source "https://rubygems.org"
      gem "rb_sys"
      gem "y-rb", git: #{repository.dump}, ref: #{revision.dump}, glob: "y-rb.gemspec"
    GEMFILE
    system(environment, Gem.ruby, "-S", "bundle", "install", chdir: directory, exception: true)
    system(environment, Gem.ruby, "-S", "bundle", "exec", "ruby", "-e", smoke, chdir: directory, exception: true)
  end
end
