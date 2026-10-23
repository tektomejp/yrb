# frozen_string_literal: true

module Y
  # Tektome in-house build: upstream y-rb Ruby API rebuilt on yrs 0.27.2, which
  # carries the apply_delete fix (upstream y-crdt@97095f2, first in yrs 0.26.0)
  # that stops deleted Pages media reappearing. 0.8.0 marks the modern-yrs build
  # and keeps it off rubygems so the app can only ever resolve it from the
  # vendored gem, never the unpatched upstream 0.7.0.
  VERSION = "0.8.2"
end
