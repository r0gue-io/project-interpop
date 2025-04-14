"use client";

import { createContext, useContext } from "react";
import { useSyncExternalStore } from "react";

declare global {
  interface Window {
    injectedWeb3?: typeof window.injectedWeb3 | undefined;
  }
}

interface InjectedWeb3Context {
  injectedWeb3: typeof window.injectedWeb3 | undefined;
}

const TestExtensionContext = createContext<InjectedWeb3Context>({
  injectedWeb3: undefined,
});

function subscribe(callback: () => void) {
  // Most extensions inject on document ready
  const documentLoadCheck = () => {
    if (document.readyState === "complete") {
      console.log("Document loaded");
      callback();
    }
  };

  // Backup check on window load
  const windowLoadCheck = () => {
    console.log("Window loaded");
    callback();
  };

  document.addEventListener("readystatechange", documentLoadCheck);
  window.addEventListener("load", windowLoadCheck);

  // Do an immediate check in case extension is already injected
  if (window.injectedWeb3) callback();

  return () => {
    document.removeEventListener("readystatechange", documentLoadCheck);
    window.removeEventListener("load", windowLoadCheck);
  };
}

// Snapshot getter function
function getSnapshot() {
  return window?.injectedWeb3;
}

// Server snapshot (for SSR)
function getServerSnapshot() {
  return undefined;
}

export function TestExtensionProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  const injectedWeb3 = useSyncExternalStore(
    subscribe,
    getSnapshot,
    getServerSnapshot
  );

  return (
    <TestExtensionContext.Provider value={{ injectedWeb3 }}>
      {children}
    </TestExtensionContext.Provider>
  );
}

export function useTestExtension() {
  const context = useContext(TestExtensionContext);
  if (!context)
    throw new Error(
      "useTestExtension must be used within TestExtensionProvider"
    );
  return context;
}
