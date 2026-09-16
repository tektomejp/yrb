# frozen_string_literal: true

source "https://rubygems.org"

# Specify your gem's dependencies in y-rb.gemspec
gem "y-rb", path: ".", glob: "y-rb.gemspec"

Bundler.load_gemspec(File.expand_path("y-rb.gemspec", __dir__)).development_dependencies.each do |dependency|
  gem dependency.name, *dependency.requirement.as_list, group: :development
end

gem "benchmark"
gem "benchmark-ips"
gem "benchmark-memory"
gem "github-markup"
gem "redcarpet"
gem "rspec"
gem "rspec-benchmark"
gem "rubocop"
gem "rubocop-performance"
gem "rubocop-rspec"
gem "yard"
