import type { Metadata } from 'next'
import './globals.css'

export const metadata: Metadata = {
  title: 'Stellar Guilds - Decentralized Guild Platform',
  description: 'Join guilds, complete bounties, and earn rewards on the Stellar network',
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  return (
    <html lang="en" className="dark">
      <body className="min-h-screen bg-stellar-navy text-stellar-white font-sans">
        {children}
      </body>
    </html>
  )
}