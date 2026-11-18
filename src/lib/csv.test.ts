import { describe, expect, it } from "vitest";
import { dailyToCsv, escapeCsvCell, sessionsToCsv, toCsv } from "./csv";
import type { SessionTotals } from "./usage";

const row: SessionTotals = {
  key: "s1",
  label: 'A "quoted", title',
  updatedAt: Date.UTC(2026, 10, 18, 12, 0, 0),
  model: "test/model",
  project: "/work/a",
  input: 10,
  output: 5,
  reasoning: 2,
  cacheRead: 100,
  cacheWrite: 0,
  fresh: 17,
  total: 117,
  messages: 3,
};

describe("escapeCsvCell", () => {
  it("leaves plain values alone", () => {
    expect(escapeCsvCell("hello")).toBe("hello");
    expect(escapeCsvCell(42)).toBe("42");
  });

  it("quotes commas, newlines and quotes", () => {
    expect(escapeCsvCell('a,b')).toBe('"a,b"');
    expect(escapeCsvCell("a\nb")).toBe('"a\nb"');
    expect(escapeCsvCell('a "b"')).toBe('"a ""b"""');
  });
});

describe("sessionsToCsv", () => {
  it("writes a header and one line per session", () => {
    const csv = sessionsToCsv([row]);
    const lines = csv.split("\n");
    expect(lines).toHaveLength(2);
    expect(lines[0]).toBe("Session,Last active,Model,Project,Messages,Input,Output,Reasoning,Cache read,Cache write,Fresh,Total");
    expect(lines[1]).toContain('"A ""quoted"", title"');
    expect(lines[1]).toContain("2026-11-18T12:00:00.000Z");
    expect(lines[1].endsWith("17,117")).toBe(true);
  });

  it("writes only the header when there is no data", () => {
    expect(sessionsToCsv([]).split("\n")).toHaveLength(1);
  });
});

describe("toCsv and dailyToCsv", () => {
  it("writes a header and escaped rows", () => {
    expect(toCsv(["A", "B"], [[1, "x,y"]])).toBe("A,B\n1,\"x,y\"");
  });

  it("writes daily token columns", () => {
    const csv = dailyToCsv([
      {
        day: "2026-11-18",
        input: 10,
        output: 5,
        reasoning: 2,
        cacheRead: 100,
        cacheWrite: 0,
        fresh: 17,
        total: 117,
      },
    ]);
    const lines = csv.split("\n");
    expect(lines[0]).toBe("Day,Fresh,Input,Output,Reasoning,Cache read,Cache write,Total");
    expect(lines[1]).toBe("2026-11-18,17,10,5,2,100,0,117");
  });
});
