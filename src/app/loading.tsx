export default function Loading() {
  return (
    <>
      <div className="page-head">
        <div>
          <div className="skeleton" style={{ width: 220, height: 36 }} />
          <div className="skeleton" style={{ width: 320, marginTop: 12 }} />
        </div>
      </div>
      <div className="grid-stats">
        {[0, 1, 2, 3].map((index) => (
          <div className="card" key={index}>
            <div className="skeleton" style={{ width: 120 }} />
            <div className="skeleton" style={{ width: 160, height: 30, marginTop: 16 }} />
          </div>
        ))}
      </div>
      <div className="card">
        <div className="skeleton" style={{ width: 180, marginBottom: 24 }} />
        <div className="skeleton" style={{ height: 240 }} />
      </div>
    </>
  );
}
