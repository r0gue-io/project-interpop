import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function trimAddress(address: string, length?: number) {
  if (!length) length = 4;
  return `${address.slice(0, length)}...${address.slice(-length)}`;
}
