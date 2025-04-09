"use client";

import { dot, pop } from "@polkadot-api/descriptors";
import type { TypedApi } from "polkadot-api";

export interface ChainConfig {
  key: string;
  name: string;
  descriptors: typeof dot;
  endpoints: string[];
  explorerUrl?: string;
}

export type AvailableApis = TypedApi<typeof dot>;

export const chainConfig: ChainConfig[] = [
  {
    key: "dot",
    name: "Polkadot",
    descriptors: dot,
    endpoints: ["wss://rpc.polkadot.io"],
  },
  {
    key: "pop",
    name: "pop",
    descriptors: pop,
    endpoints: ["ws://localhost:9944"],
  },
];
