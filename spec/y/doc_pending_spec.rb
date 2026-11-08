# frozen_string_literal: true

require "spec_helper"

RSpec.describe Y::Doc do
  let(:inserted_status) { { inserted: true, deleted: false, pending: false } }
  let(:deleted_status) { { inserted: false, deleted: true, pending: false } }
  let(:unchanged_status) { { inserted: false, deleted: false, pending: false } }
  let(:waiting_status) { { inserted: false, deleted: false, pending: true } }

  def edits
    doc = Y::Doc.new(17)
    text = doc.get_text("body")
    text << "Door "
    first = doc.diff
    before = doc.state
    text << "850"
    [first, doc.diff(before)]
  end

  def deletion_edits
    doc = described_class.new(17)
    text = doc.get_text("body")
    text << "Door 850"
    first = doc.diff
    before = doc.state
    text.slice!(5, 3)
    [first, doc.diff(before)]
  end

  def restore(waiting, first)
    restored = described_class.new
    restored.sync(waiting.encode_state_as_update)
    restored.sync(first)
    restored
  end

  def map_edits_with_gap
    source = described_class.new(17)
    map = source.get_map("m")
    updates = []
    source.attach { |bytes| updates << bytes }
    %w[a b c].each_with_index { |key, index| map[key] = index }
    updates.drop(1)
  end

  it "reports independent insertions after a missing clock range" do
    doc = described_class.new
    results = map_edits_with_gap.map { |bytes| doc.apply_with_status(bytes) }
    expect([results.map { |result| result[:inserted] }, doc.get_map("m")["c"]])
      .to eq([[true, true], 2])
  end

  it "keeps a waiting insertion in a complete encoding" do
    first, second = edits
    waiting = described_class.new
    waiting.sync(second)
    pending = waiting.pending?
    restored = restore(waiting, first)
    expect([pending, restored.get_text("body").to_s, restored.pending?])
      .to eq([true, "Door 850", false])
  end

  it "keeps a waiting deletion in a complete encoding" do
    first, deletion = deletion_edits
    waiting = described_class.new
    waiting.sync(deletion)
    pending = waiting.pending?
    restored = restore(waiting, first)
    expect([pending, restored.get_text("body").to_s, restored.pending?])
      .to eq([true, "Door ", false])
  end

  it "preserves integrated content alongside waiting content" do
    first, second = edits
    waiting = described_class.new(23)
    waiting.get_text("other") << "kept"
    waiting.sync(second)
    restored = restore(waiting, first)
    expect([restored.get_text("other").to_s, restored.get_text("body").to_s])
      .to eq(["kept", "Door 850"])
  end

  it "reports applied insertions and duplicate updates separately" do
    first, = edits
    doc = described_class.new
    results = [doc.apply_with_status(first), doc.apply_with_status(first)]
    expect(results).to eq([inserted_status, unchanged_status])
  end

  it "reports waiting content without calling it an insertion" do
    first, second = edits
    doc = described_class.new
    statuses = [doc.apply_with_status(second), doc.apply_with_status(first)]
    expect([statuses, doc.get_text("body").to_s])
      .to eq([[waiting_status, inserted_status], "Door 850"])
  end

  it "reports new deletions and duplicate deletions separately" do
    first, deletion = deletion_edits
    doc = described_class.new
    doc.sync(first)
    expect([doc.apply_with_status(deletion), doc.apply_with_status(deletion)])
      .to eq([{ inserted: false, deleted: true, pending: false },
              { inserted: false, deleted: false, pending: false }])
  end

  it "reports a waiting delete and the later deletion it causes" do
    first, deletion = deletion_edits
    doc = described_class.new
    expect([doc.apply_with_status(deletion), doc.apply_with_status(first)])
      .to eq([{ inserted: false, deleted: false, pending: true },
              { inserted: true, deleted: true, pending: false }])
  end

  it "exposes insertion ranges that begin after clock zero" do
    _, second = edits
    update = Y::Update.new(second)
    expect([update.insertions, update.deletions]).to eq([[[17, 5, 8]], []])
  end

  it "exposes deletion ranges without treating them as insertions" do
    _, deletion = deletion_edits
    update = Y::Update.new(deletion)
    expect([update.insertions, update.deletions]).to eq([[], [[17, 5, 8]]])
  end

  it "rejects malformed update metadata" do
    expect { Y::Update.new([255]) }
      .to raise_error(RuntimeError, /cannot decode update/)
  end

  it "rejects malformed update application" do
    expect { described_class.new.apply_with_status([255]) }
      .to raise_error(RuntimeError, /cannot decode update/)
  end

  it "keeps clean state compatible with the existing diff encoding" do
    first, = deletion_edits
    doc = described_class.new
    doc.sync(first)
    expect(doc.encode_state_as_update).to eq(doc.diff)
  end

  it "reports each apply within an already open transaction" do
    first, deletion = deletion_edits
    doc = described_class.new
    statuses = doc.transact do
      [first, deletion, deletion].map { |bytes| doc.apply_with_status(bytes) }
    end
    expect(statuses).to eq([
                             { inserted: true, deleted: false, pending: false },
                             { inserted: false, deleted: true, pending: false },
                             { inserted: false, deleted: false, pending: false }
                           ])
  end
end
