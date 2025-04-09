"use client";

import { ThemeProvider } from "./theme-provider";
import { PolkadotExtensionProvider } from "./polkadot-extension-provider";
import { ChainProvider } from "./chain-provider";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { TxProvider } from "./tx-provider";

export function Providers({ children }: { children: React.ReactNode }) {
  const queryClient = new QueryClient();
  return (
    <ThemeProvider defaultTheme="dark">
      <QueryClientProvider client={queryClient}>
        <PolkadotExtensionProvider>
          <ChainProvider>
            <TxProvider>{children}</TxProvider>
          </ChainProvider>
        </PolkadotExtensionProvider>
      </QueryClientProvider>
    </ThemeProvider>
  );
}
