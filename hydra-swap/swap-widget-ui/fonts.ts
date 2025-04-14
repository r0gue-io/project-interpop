import { Geist_Mono, Unbounded, Manrope } from "next/font/google";

export const fontSans = Manrope({
  variable: "--font-sans",
  subsets: ["latin"],
  display: "swap",
  weight: "400",
});

export const fontUnbounded = Unbounded({
  variable: "--font-unbounded",
  subsets: ["latin"],
  display: "swap",
});

export const fontMono = Geist_Mono({
  variable: "--font-mono",
  subsets: ["latin"],
});
