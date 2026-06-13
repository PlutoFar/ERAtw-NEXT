import Ajv2020 from "ajv/dist/2020";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { mockMapModel, mockSystemStatus } from "./engine/mockData";

const here = dirname(fileURLToPath(import.meta.url));
const schemaDir = resolve(here, "../../../schemas");
const loadSchema = (file: string): object =>
  JSON.parse(readFileSync(resolve(schemaDir, file), "utf8"));

const compileSchema = (file: string) =>
  new Ajv2020({ allErrors: true, strict: false }).compile(loadSchema(file));

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
});
