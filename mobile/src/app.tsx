const launchChecklist = [
  "Shared Rust core",
  "Local encrypted data",
  "Foreground and resume sync"
];

export function App() {
  return (
    <main className="mobile-shell">
      <header className="app-header">
        <span className="eyebrow">Aether</span>
        <h1>Your day, held together.</h1>
        <p>
          The mobile shell is ready for the shared core. Daily journal capture,
          tasks, and sync will arrive here without duplicating the desktop data model.
        </p>
      </header>

      <section aria-labelledby="launch-readiness" className="readiness-card">
        <div className="section-heading">
          <span className="status-dot" aria-hidden="true" />
          <h2 id="launch-readiness">Mobile foundation</h2>
        </div>
        <ul>
          {launchChecklist.map((item) => (
            <li key={item}>{item}</li>
          ))}
        </ul>
      </section>

      <button className="primary-action" type="button">
        Start with today
      </button>
    </main>
  );
}
