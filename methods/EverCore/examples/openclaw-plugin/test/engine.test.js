import assert from "node:assert/strict";
import test from "node:test";

import { createContextEngine } from "../src/engine.js";

const pluginMeta = {
  id: "evermind-ai-everos",
  name: "EverOS Test Engine",
  version: "0.0.0-test",
};

function createTestEngine() {
  const logger = {
    info: () => {},
    warn: () => {},
  };
  return createContextEngine(pluginMeta, {}, logger);
}

test("passive memory engine does not expose a compact capability", () => {
  const engine = createTestEngine();

  assert.equal(engine.info.ownsCompaction, false);
  assert.equal(Object.hasOwn(engine, "compact"), false);
});

