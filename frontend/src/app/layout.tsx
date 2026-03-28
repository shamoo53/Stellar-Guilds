import type { Metadata } from "next";
import { ThemeProvider } from "next-themes";

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
        <html lang="en" suppressHydrationWarning>
            <body>
                {/* next-themes persists preference to localStorage under key "theme" by default */}
                <ThemeProvider
                    attribute="class"
                    defaultTheme="system"
                    enableSystem
                    disableTransitionOnChange
                >
                    {children}
                </ThemeProvider>
            </body>
        </html>
    );
}
