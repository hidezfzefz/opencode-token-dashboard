"use client";

import { useRouter } from "next/navigation";
import { RefreshCw } from "lucide-react";
import { useTransition } from "react";

export function RefreshButton() {
  const router = useRouter();
  const [pending, startTransition] = useTransition();

  return (
    <button
      type="button"
      className="btn"
      onClick={() => startTransition(() => router.refresh())}
      disabled={pending}
    >
      <RefreshCw size={15} strokeWidth={1.5} className={pending ? "spin" : undefined} aria-hidden="true" />
      Refresh
    </button>
  );
}
