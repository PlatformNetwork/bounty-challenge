// src/AppCore.tsx
import React, { Suspense, lazy, useEffect, useState } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { ErrorBoundary } from 'react-error-boundary';

// Lazy load components that might use Tauri APIs
const Welcome = lazy(() => import('./pages/Welcome'));
const Dashboard = lazy(() => import('./pages/Dashboard'));

// Safe Tauri API wrapper
const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;

const getCurrentWebviewWindow = () => {
  if (isTauri) {
    // Dynamic import to avoid static dependency
    return import('@tauri-apps/api/webviewWindow').then(mod => mod.getCurrentWebviewWindow());
  }
  return null;
};

const AppCore: React.FC = () => {
  const [tauriWindow, setTauriWindow] = useState<any>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const init = async () => {
      try {
        if (isTauri) {
          const window = await getCurrentWebviewWindow();
          setTauriWindow(window);
        }
      } catch (err) {
        console.warn('Tauri API not available, running in browser mode');
        setTauriWindow(null);
      }
    };
    init();
  }, []);

  if (error) {
    return (
      <div className="error-screen">
        <h2>Failed to Initialize</h2>
        <p>{error}</p>
        <button onClick={() => window.location.reload()}>Retry</button>
      </div>
    );
  }

  return (
    <ErrorBoundary
      fallback={<div className="error-screen">
        <h2>Failed to Initialize</h2>
        <p>An unexpected error occurred</p>
        <button onClick={() => window.location.reload()}>Retry</button>
      </div>}
      onError={(err) => {
        console.error('AppCore Error:', err);
        setError(err.message);
      }}
    >
      <BrowserRouter>
        <Suspense fallback={<div className="loading-screen">Loading...</div>}>
          <Routes>
            <Route path="/" element={<Welcome />} />
            <Route path="/dashboard" element={<Dashboard />} />
          </Routes>
        </Suspense>
      </BrowserRouter>
    </ErrorBoundary>
  );
};

export default AppCore;
