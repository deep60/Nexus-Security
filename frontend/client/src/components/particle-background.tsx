export function ParticleBackground() {
  return (
    <>
      {/* Particle Background */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none z-0">
        {Array.from({ length: 9 }).map((_, i) => (
          <div
            key={i}
            className="particle"
            style={{
              left: `${(i + 1) * 10}%`,
              animationDelay: `${i * 2}s`,
            }}
          />
        ))}
      </div>

      {/* Cyber Grid Background */}
      <div className="fixed inset-0 cyber-grid opacity-30 pointer-events-none z-0" />
    </>
  );
}
