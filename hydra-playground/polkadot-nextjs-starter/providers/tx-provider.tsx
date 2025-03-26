import { createContext, useContext, useState } from "react";

interface TxContextType {
  isProcessing: boolean;
  setIsProcessing: (isProcessing: boolean) => void;
}

export const TxContext = createContext<TxContextType>({
  isProcessing: false,
  setIsProcessing: () => {},
});

export const TxProvider = ({ children }: { children: React.ReactNode }) => {
  const [isProcessing, setIsProcessing] = useState(false);

  return (
    <TxContext.Provider value={{ isProcessing, setIsProcessing }}>
      {children}
    </TxContext.Provider>
  );
};

export function useTxContext() {
  const context = useContext(TxContext);
  if (!context) throw new Error("useTx must be used within TxProvider");
  return context;
}
