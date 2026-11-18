import type { Metadata } from "next";
import type { ReactNode } from "react";
import "./globals.css";
import { TopBar } from "@/components/top-bar";

// Usage data is read live from a local database on every request, so navigations
// intentionally wait for the server. Opt out of instant-navigation validation.
export const instant = false;

export const metadata: Metadata = {
  title: "Token usage — OpenCode dashboard",
  description: "A local dashboard for OpenCode token usage.",
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en">
      <body>
        <TopBar />
        <main className="shell page">{children}</main>
      </body>
    </html>
  );
}
