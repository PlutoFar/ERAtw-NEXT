import Ajv2020 from "ajv/dist/2020";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { mockMapModel, mockSystemStatus } from "./engine/mockData";

const here = dirname(fileURLToPath(import.meta.url));
const schemaDir = resolve(here, "../../../schemas");
const loadJson = (...segments: string[]): object =>
  JSON.parse(readFileSync(resolve(schemaDir, ...segments), "utf8"));
const loadSchema = (file: string): object => loadJson(file);
const loadM1Fixture = (file: string): object => loadJson("fixtures/m1", file);
const loadM2Fixture = (file: string): object => loadJson("fixtures/m2", file);
const loadM3Fixture = (file: string): object => loadJson("fixtures/m3", file);
const loadM4Fixture = (file: string): object => loadJson("fixtures/m4", file);

const compileSchema = (file: string) => {
  const ajv = new Ajv2020({ allErrors: true, strict: false });
  if (file !== "game-state.schema.json") {
    ajv.addSchema(loadSchema("game-state.schema.json"));
  }
  if (file !== "game-command.schema.json") {
    ajv.addSchema(loadSchema("game-command.schema.json"));
  }
  return ajv.compile(loadSchema(file));
};

const m2Fixtures = [
  ["content-package.schema.json", "manifest.valid.json"],
  ["content-dictionary-entry.schema.json", "dictionary-entry.valid.json"],
  ["content-character.schema.json", "character.valid.json"],
  ["content-resource.schema.json", "resource.valid.json"],
  ["content-location.schema.json", "location.valid.json"],
  ["content-dialogue-source.schema.json", "dialogue-source.valid.json"],
  ["content-dialogue-scene.schema.json", "dialogue-scene.valid.json"],
  ["content-source-file.schema.json", "source-file.valid.json"],
  ["content-unmapped-item.schema.json", "unmapped-item.valid.json"],
  ["content-package-validation.schema.json", "validation-report.valid.json"],
  ["migration-report.schema.json", "migration-report.valid.json"],
] as const;

const m1Fixtures = [
  ["content-audit-summary.schema.json", "content-audit-summary.valid.json"],
  ["content-audit-file-record.schema.json", "file-record.valid.json"],
  ["content-audit-directory-record.schema.json", "directory-record.valid.json"],
  ["content-audit-erb-stats.schema.json", "erb-stats.valid.json"],
  ["content-audit-csv-stats.schema.json", "csv-stats.valid.json"],
  ["content-audit-resources.schema.json", "resources.valid.json"],
] as const;

const m3Fixtures = [
  ["content-package-index.schema.json", "content-package-index.valid.json"],
] as const;

const m4Fixtures = [
  ["game-command.schema.json", "game-command.valid.json"],
  ["game-state.schema.json", "game-state.valid.json"],
  ["save-envelope.schema.json", "save-envelope.valid.json"],
  ["save-report.schema.json", "save-report.valid.json"],
] as const;

describe("JSON Schema 契约", () => {
  it("system-status fixture 符合 system-status/v1", () => {
    const validate = compileSchema("system-status.schema.json");
    const ok = validate(mockSystemStatus);
    expect(validate.errors ?? []).toEqual([]);
    expect(ok).toBe(true);
  });

  it("map-model fixture 符合 map-model/v1", () => {
    const validate = compileSchema("map-model.schema.json");
    const ok = validate(mockMapModel);
    expect(validate.errors ?? []).toEqual([]);
    expect(ok).toBe(true);
  });

  it("拒绝缺失必填字段的状态", () => {
    const validate = compileSchema("system-status.schema.json");
    const broken = { ...mockSystemStatus } as Record<string, unknown>;
    delete broken.engine;
    expect(validate(broken)).toBe(false);
  });

  it.each(m1Fixtures)("M1 fixture %s 符合 schema", (schemaFile, fixtureFile) => {
    const validate = compileSchema(schemaFile);
    const ok = validate(loadM1Fixture(fixtureFile));
    expect(validate.errors ?? []).toEqual([]);
    expect(ok).toBe(true);
  });

  it.each(m2Fixtures)("M2 fixture %s 符合 schema", (schemaFile, fixtureFile) => {
    const validate = compileSchema(schemaFile);
    const ok = validate(loadM2Fixture(fixtureFile));
    expect(validate.errors ?? []).toEqual([]);
    expect(ok).toBe(true);
  });

  it.each(m3Fixtures)("M3 fixture %s 符合 schema", (schemaFile, fixtureFile) => {
    const validate = compileSchema(schemaFile);
    const ok = validate(loadM3Fixture(fixtureFile));
    expect(validate.errors ?? []).toEqual([]);
    expect(ok).toBe(true);
  });

  it.each(m4Fixtures)("M4 fixture %s 符合 schema", (schemaFile, fixtureFile) => {
    const validate = compileSchema(schemaFile);
    const ok = validate(loadM4Fixture(fixtureFile));
    expect(validate.errors ?? []).toEqual([]);
    expect(ok).toBe(true);
  });

  it("M2 character draft 必须携带 sourceTrace", () => {
    const validate = compileSchema("content-character.schema.json");
    const broken = { ...loadM2Fixture("character.valid.json") } as Record<string, unknown>;
    delete broken.sourceTrace;
    expect(validate(broken)).toBe(false);
  });
});
