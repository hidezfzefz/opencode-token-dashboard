"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { Activity } from "lucide-react";

const links = [
  { href: "/", label: "Overview" },
  { href: "/sessions", label: "Sessions" },
  { href: "/models", label: "Models" },
  { href: "/projects", label: "Projects" },
];

export function TopBar() {
  const pathname = usePathname();

  return (
    <header className="topbar">
      <div className="shell topbar-inner">
        <Link href="/" className="wordmark">
          <span className="wordmark-mark" aria-hidden="true">
            <Activity size={15} strokeWidth={1.5} />
          </span>
          token usage
        </Link>
        <nav className="nav" aria-label="Main navigation">
          {links.map((link) => (
            <Link
              key={link.href}
              href={link.href}
              className="nav-link"
              data-active={pathname === link.href}
              aria-current={pathname === link.href ? "page" : undefined}
            >
              {link.label}
            </Link>
          ))}
        </nav>
      </div>
    </header>
  );
}
