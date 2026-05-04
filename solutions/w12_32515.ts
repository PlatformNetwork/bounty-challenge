// src/AppCore.tsx
import React, { Suspense, lazy, useEffect, useState } from 'react';
import { useLocation } from 'react-router-dom';

// Lazy load the main app content
const AppContent = lazy(() => import('./AppContent'));

// Check if running in Tauri environment
const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;

const AppCore: React.FC = () => {
  const [isInitialized, setIsInitialized] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const location = useLocation();

  useEffect(() => {
    const initializeApp = async () => {
      try {
        // Only attempt Tauri initialization if in Tauri environment
        if (isTauri) {
          const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
          const appWindow = getCurrentWebviewWindow();
          // Perform any Tauri-specific initialization here
          await appWindow.show();
        }
        setIsInitialized(true);
      } catch (err) {
        console.error('Failed to initialize app:', err);
        setError(err instanceof Error ? err.message : 'Unknown initialization error');
      }
    };

    initializeApp();
  }, []);

  if (error) {
    return (
      <div className="failed-initialize">
        <h1>Failed to Initialize</h1>
        <p>{error}</p>
        <button onClick={() => window.location.reload()}>Retry</button>
      </div>
    );
  }

  if (!isInitialized) {
    return (
      <div className="loading">
        <div className="spinner" />
        <p>Initializing...</p>
      </div>
    );
  }

  return (
    <Suspense fallback={<div className="loading">Loading...</div>}>
      <AppContent />
    </Suspense>
  );
};

export default AppCore;
