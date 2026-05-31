import { describe, expect, it } from "vitest";
import datasetExport from "./datasets";

describe("datasetExport", () => {
  it("lists the Marchantia assembly", () => {
    expect.assertions(1);

    expect(datasetExport.datasets).toEqual([
      {
        assembly: "MpTak1_v7.1",
        species: "Marchantia polymorpha",
        status: "Available",
      },
    ]);
  });
});
