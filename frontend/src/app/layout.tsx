import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import RouteProgressBar from "../features/profile/components/RouteProgressBar";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "Stellar Guilds",
  description: "User Profile & Reputation Dashboard",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <body className={inter.className}>
        <RouteProgressBar />
        <div className="min-h-screen bg-stellar-navy text-stellar-white font-sans">
             {children}
        </div>
      </body>
    </html>
  );
}
