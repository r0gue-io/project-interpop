"use client";
import { fontUnbounded } from "@/fonts";
import { cn } from "@/lib/utils";
import React, { useState } from "react";
import { SS58String } from "polkadot-api";
import { getInkClient } from "@polkadot-api/ink-contracts";
import { contracts } from "@polkadot-api/descriptors";
import { useChain } from "@/providers/chain-provider";
import { usePolkadotExtension } from "@/providers/polkadot-extension-provider";
import { createInkSdk } from "@polkadot-api/sdk-ink";

const fundOnHydraContract = getInkClient(contracts.execute_on_hydra);

const FundOnHydra: React.FC = () => {
  const [contractAddress, setContractAddress] = useState<string>("");
  const [amount, setAmount] = useState<string>("1");
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const { api } = useChain();
  const { selectedAccount } = usePolkadotExtension();

  const handleFund = async () => {
    if (!api) {
      alert("Expect API is initialized");
      return;
    }
    if (!selectedAccount) {
      alert("Please select an account");
      return;
    }
    if (!contractAddress) {
      alert("Please enter a contract address");
      return;
    }

    if (!amount || parseInt(amount) <= 0) {
      alert("Please enter a valid amount");
      return;
    }

    setIsLoading(true);
    try {
      // This is where you would integrate PAPI to interact with the ink! smart contract
      console.log(
        `Funding contract at address: ${contractAddress} with amount: ${amount}`,
      );

      const fundOnHydraSdk = createInkSdk(api, contracts.execute_on_hydra);
      const fund_on_hydra = fundOnHydraContract.message(
        "create_pop_to_hydra_xcm",
      );

      fund_on_hydra.encode({
        amount: BigInt(amount),
        ref_time: BigInt(10000000000), // TODO: make param
        proof_size: BigInt(1000000),
      });
      const contract = fundOnHydraSdk.getContract(contractAddress);

      console.log(selectedAccount?.address as SS58String);
      await contract
        .send("create_pop_to_hydra_xcm", {
          origin: selectedAccount?.address as SS58String,
          data: {
            amount: BigInt(amount),
            ref_time: BigInt(10000000000), // TODO: make param
            proof_size: BigInt(1000000),
          },
        })
        .signAndSubmit(selectedAccount?.polkadotSigner);

      alert(`Contract funded successfully with ${amount} tokens!`);
    } catch (error) {
      console.error("Error funding contract:", error);
      alert("Failed to fund contract");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div
      style={{
        padding: "16px",
        border: "1px solid #ccc",
        borderRadius: "8px",
        maxWidth: "400px",
      }}
    >
      <h3 style={{ marginBottom: "12px" }}>Fund Contract on Hydra</h3>

      <div style={{ marginBottom: "16px" }}>
        <input
          type="text"
          placeholder="Contract Address"
          value={contractAddress}
          onChange={(e) => setContractAddress(e.target.value)}
          style={{
            width: "100%",
            padding: "8px",
            borderRadius: "4px",
            border: "1px solid #ccc",
          }}
        />
      </div>

      <div style={{ marginBottom: "16px" }}>
        <input
          type="number"
          placeholder="Amount"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          min="0"
          step="0.1"
          style={{
            width: "100%",
            padding: "8px",
            borderRadius: "4px",
            border: "1px solid #ccc",
          }}
        />
      </div>

      <button
        onClick={handleFund}
        disabled={
          isLoading || !contractAddress || !amount || parseFloat(amount) <= 0
        }
        style={{
          width: "100%",
          padding: "10px",
          backgroundColor: isLoading ? "#cccccc" : "#3182ce",
          color: "white",
          border: "none",
          borderRadius: "4px",
          cursor:
            isLoading || !contractAddress || !amount || parseFloat(amount) <= 0
              ? "not-allowed"
              : "pointer",
        }}
      >
        {isLoading ? "Processing..." : "Fund"}
      </button>
    </div>
  );
};

export default function Home() {
  return (
    <main className="flex min-h-screen p-8 pb-20 flex-col gap-[32px] row-start-2 items-center justify-center relative">
      <h1
        className={cn(
          "text-6xl bg-clip-text text-transparent bg-gradient-to-r from-foreground/70 via-foreground to-foreground/70",
          fontUnbounded.className,
        )}
      >
        Polkadot Next.js Starter
      </h1>
      <p>A starter project for building a Polkadot dApp with Next.js.</p>
      <FundOnHydra />
    </main>
  );
}
