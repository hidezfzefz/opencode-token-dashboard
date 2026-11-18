import Link from "next/link";

export default function NotFound() {
  return (
    <section className="card">
      <h1 className="page-title">Page not found</h1>
      <p className="page-subtitle">There is no dashboard page at this address.</p>
      <Link href="/" className="btn btn-primary" style={{ marginTop: 24 }}>
        Back to overview
      </Link>
    </section>
  );
}
