// src/AppCore.tsx
import React, { Suspense, lazy, useEffect, useState } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { ErrorBoundary } from 'react-error-boundary';

// Lazy load components
const Welcome = lazy(() => import('./pages/Welcome'));
const Dashboard = lazy(() => import('./pages/Dashboard'));
const Settings = lazy(() => import('./pages/Settings'));

// Check if running in Tauri environment
const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;

// Fallback component for loading state
const LoadingFallback = () => (
  <div className="flex items-center justify-center h-screen">
    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
  </div>
);

// Error fallback component
const ErrorFallback = ({ error, resetErrorBoundary }: { error: Error; resetErrorBoundary: () => void }) => (
  <div className="flex flex-col items-center justify-center h-screen">
    <h2 className="text-2xl font-bold mb-4">Failed to Initialize</h2>
    <p className="text-red-500 mb-4">{error.message}</p>
    <button
      onClick={resetErrorBoundary}
      className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
    >
      Try Again
    </button>
  </div>
);

// Safe Tauri API wrapper
const safeTauriAPI = {
  getCurrentWebviewWindow: () => {
    if (isTauri) {
      try {
        // Dynamic import to avoid crashing in browser
        const { getCurrentWebviewWindow } = require('@tauri-apps/api/webviewWindow');
        return getCurrentWebviewWindow();
      } catch {
        return null;
      }
    }
    return null;
  },
  getCurrentWindow: () => {
    if (isTauri) {
      try {
        const { getCurrentWindow } = require('@tauri-apps/api/window');
        return getCurrentWindow();
      } catch {
        return null;
      }
    }
    return null;
  }
};

// Tauri-specific component wrapper
const TauriWrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [tauriReady, setTauriReady] = useState(!isTauri);

  useEffect(() => {
    if (isTauri) {
      // Wait for Tauri to be ready
      const checkTauri = setInterval(() => {
        if (window.__TAURI__) {
          clearInterval(checkTauri);
          setTauriReady(true);
        }
      }, 100);

      // Timeout after 5 seconds
      setTimeout(() => {
        clearInterval(checkTauri);
        setTauriReady(true);
      }, 5000);

      return () => clearInterval(checkTauri);
    }
  }, []);

  if (!tauriReady) {
    return <LoadingFallback />;
  }

  return <>{children}</>;
};

// Main App Component
const AppCore: React.FC = () => {
  const [error, setError] = useState<Error | null>(null);

  const handleError = (error: Error) => {
    console.error('App Error:', error);
    setError(error);
  };

  if (error) {
    return (
      <ErrorFallback
        error={error}
        resetErrorBoundary={() => setError(null)}
      />
    );
  }

  return (
    <ErrorBoundary
      FallbackComponent={ErrorFallback}
      onError={handleError}
    >
      <TauriWrapper>
        <BrowserRouter>
          <Suspense fallback={<LoadingFallback />}>
            <Routes>
              <Route path="/" element={<Welcome />} />
              <Route path="/dashboard" element={<Dashboard />} />
              <Route path="/settings" element={<Settings />} />
            </Routes>
          </Suspense>
        </BrowserRouter>
      </TauriWrapper>
    </ErrorBoundary>
  );
};

export default AppCore;
