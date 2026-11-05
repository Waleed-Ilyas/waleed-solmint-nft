import type { Metadata } from "next";
import "./styles.css";

export const metadata: Metadata = {
  title: "SolMint — Marketplace Interface Showcase",
  description: "A local NFT marketplace concept with illustrative sample items.",
  robots: { index: false, follow: false },
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
