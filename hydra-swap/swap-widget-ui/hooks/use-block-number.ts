"use client";

import { useChain } from "@/providers/chain-provider";
import { useEffect, useState } from "react";

export function useBlockNumber() {
  const { client } = useChain();
  const [blockNumber, setBlockNumber] = useState<number | null>(null);

  useEffect(() => {
    if (!client) return;
    const subscription = client.finalizedBlock$.subscribe((value) => {
      setBlockNumber(value.number);
    });
    return () => subscription?.unsubscribe();
  }, [client]);

  return blockNumber;
}
