"use client";

import { popTestnetLocal } from "@polkadot-api/descriptors";
import type { TypedApi } from "polkadot-api";

export interface ChainConfig {
  key: string;
  name: string;
  descriptors: typeof popTestnetLocal;
  endpoints: string[];
  explorerUrl?: string;
}

export type AvailableApis = TypedApi<typeof popTestnetLocal>;

export const chainConfig: ChainConfig[] = [
  {
    key: "popTestnetLocal",
    name: "popTestnetLocal",
    descriptors: popTestnetLocal,
    endpoints: ["ws://localhost:9944"],
  },
];
