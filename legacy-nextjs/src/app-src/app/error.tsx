"use client";

import { useEffect } from "react";

export default function Error({ error, reset }: { error: Error & { digest?: string }; reset: () => void }) {
  useEffect(() => {
    console.error(error);
  }, [error]);

  return (
    <section className="card">
      <h1 className="page-title">Could not load token usage</h1>
      <p className="page-subtitle">{error.message || "An unexpected error happened while reading the database."}</p>
      <p className="stat-meta">
        Check that the OpenCode database exists and is readable. Set OPENCODE_DB_PATH to use a different file.
      </p>
      <button className="btn btn-primary" type="button" onClick={reset} style={{ marginTop: 24 }}>
        Try again
      </button>
    </section>
  );
}
