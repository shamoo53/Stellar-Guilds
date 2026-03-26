import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "../globals.css";
import { getMessages } from 'next-intl/server';
import { NextIntlClientProvider } from 'next-intl';
import { locales, defaultLocale, type Locale } from '@/i18n';
import { ErrorBoundary } from '@/components/ErrorBoundary';

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "Stellar Guilds",
  description: "User Profile & Reputation Dashboard",
};

export default async function RootLayout({
  children,
  params,
}: {
  children: React.ReactNode;
  params: Promise<{ locale: string }>;
}) {
  const resolvedParams = await params;
  let locale = resolvedParams.locale;

  // Validate that the incoming locale is supported
  if (!locales.includes(locale as Locale)) {
    locale = defaultLocale;
  }

  // Get the messages for the current locale
  const messages = await getMessages();

  return (
    <html lang={locale} className="dark">
      <body className={inter.className}>
        <NextIntlClientProvider locale={locale} messages={messages}>
          <ErrorBoundary>
            <div className="min-h-screen flex flex-col bg-stellar-navy text-stellar-white font-sans">
              {children}
            </div>
          </ErrorBoundary>
        </NextIntlClientProvider>
      </body>
    </html>
  );
}

// Generate static params for all supported locales
export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}