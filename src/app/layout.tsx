import type { Metadata } from "next";
import "./globals.css";
import { TopBar } from "@/components/top-bar";

export const metadata: Metadata = {
  title: "Token usage — OpenCode dashboard",
  description: "A local dashboard for OpenCode token usage.",
};

export default function RootLayout({ children }: LayoutProps<"/">) {
  return (
    <html lang="en">
      <body>
        <TopBar />
        <main className="shell page">{children}</main>
      </body>
    </html>
  );
}
