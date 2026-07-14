<!-- FORK NOTICE -->
> ## ⚠️ Downstream fork — not the upstream project
>
> Tektome's fork of [`y-crdt/yrb`](https://github.com/y-crdt/yrb), published as **`v0.8.0`**.
>
> **Why it exists:** the newest upstream release (`v0.7.0`) is pinned to `yrs 0.17.4`, which has an in-store range-delete bug — deleted Pages media reappears after reload (fixed upstream in `yrs 0.26.0`, commit `y-crdt/y-crdt@97095f2`). No released `y-rb` targets a modern `yrs`, so we maintain our own.
>
> **What changed vs upstream `v0.7.0`:** the Ruby bindings (`ext/yrb`) are rebuilt on **stock `yrs 0.27.2`** from crates.io. That meant porting the bindings to the newer `yrs` API (`apply_update` now returns a `Result`; awareness moved from the separate `y-sync` crate into `yrs::sync`; `Subscription` is no longer generic) and dropping the `y-sync` dependency. We use the published `yrs` crate as-is — no `yrs` fork.
>
> **Version `0.8.0`:** a minor bump above upstream `0.7.0`, marking the modern-`yrs` rebuild and keeping it above the unpatched upstream gem.
>
> **Consumed by:** the Tektome `computer` app, which vendors the prebuilt gem (see that repo's `docs/pages-yrs-vendored-gem.md`).
>
> **Upstreamable:** this is a clean forward port and could be offered back to `y-crdt/yrb` as a pull request.
<!-- END FORK NOTICE -->

<p align="center">
  <img alt="Yrb" src="./docs/assets/logo.png" width="300" />
</p>

---

# yrb

> Ruby bindings for yrs ("wires")

Yrb is a Ruby binding for Y-CRDT. It provides distributed data types that enable
real-time collaboration between devices. Yrb can sync data with any other
platform that has a Y-CRDT binding, allowing for seamless cross-domain
communication.

The library is a thin wrapper around Yrs, taking advantage of the safety and
performance of Rust.

## Installation

Add this line to your application's Gemfile:

```ruby
gem 'y-rb'
```

And then execute:

    $ bundle install

Or install it yourself as:

    $ gem install y-rb

### Required RubyGems version

If you want to use `y-rb` on `x86_64-linux-musl` systems, you most likely need
to update your system RubyGems due to a
[bug in RubyGems < 3.3.21](https://github.com/rubygems/rubygems/pull/5852). For
older RubyGems versions, the platform is described as `x86_64-linux-musl`, but
`x86_64-linux` is fetched instead. 

```bash
gem update --system
```

## Usage

```ruby
# creates a new document and text structure
local = Y::Doc.new 
local_text = local.get_text("my text")

# add some data to the text structure
local_text << "hello"  
  
# create a remote document sharing the same text structure
remote = Y::Doc.new 
remote_text = remote.get_text("my text")  

# retrieve the current state of the remote document
remote_state = remote.state  

# create an update for the remote document based on the current
# state of the remote document
update = local.diff(remote_state)  
  
# apply update to remote document
remote.sync(update)  

puts remote_text.to_s == local_text.to_s # true  
```  

More [examples](docs/examples.md).

## Development

Make sure you have `cargo` available (2021 edition). The gem needs the lib to
be built every time when there is a change.

```bash
cargo build --release
```

After checking out the repo, run `bin/setup` to install dependencies. Then,
run `rake spec` to run the tests. You can also run `bin/console` for an
interactive prompt that will allow you to experiment.

To install this gem onto your local machine, run `bundle exec rake install`.
To release a new version, update the version number in `version.rb`, and then
run `bundle exec rake release`, which will create a git tag for the version,
push git commits and the created tag, and push the `.gem` file to
[rubygems.org](https://rubygems.org).

## Docs

You can run `yard` locally and open the docs with:

```bash
yard server 
open "http://0.0.0.0:8808/"
```

## Decision log

For this `gem`, we maintain a [decision log](docs/decisions.md). Please consult it
in case there is some ambiguity in terms of why certain implementation details
look as they are. 

## License

The gem is available as open source under the terms of the
[MIT License](https://opensource.org/licenses/MIT).
