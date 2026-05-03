// src/AppCore.tsx
import React, { Suspense, lazy, useEffect, useState } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { ErrorBoundary } from 'react-error-boundary';

// Lazy load components
const Welcome = lazy(() => import('./pages/Welcome'));
const Dashboard = lazy(() => import('./pages/Dashboard'));
const Settings = lazy(() => import('./pages/Settings'));

// Safe Tauri API check
const isTauri = typeof window !== 'undefined' && 
  window.__TAURI_INTERNALS__ !== undefined && 
  window.__TAURI_INTERNALS__.metadata !== undefined;

// Fallback component for non-Tauri environments
const NonTauriFallback: React.FC = () => {
  const [isReady, setIsReady] = useState(false);

  useEffect(() => {
    // Simulate initialization delay
    const timer = setTimeout(() => setIsReady(true), 1000);
    return () => clearTimeout(timer);
  }, []);

  if (!isReady) {
    return (
      <div className="loading-screen">
        <div className="spinner" />
        <p>Initializing...</p>
      </div>
    );
  }

  return (
    <div className="welcome-ui">
      <h1>Welcome to Cortex</h1>
      <p>Running in browser mode</p>
      <button onClick={() => window.location.href = '/dashboard'}>
        Get Started
      </button>
    </div>
  );
};

// Error fallback component
const ErrorFallback: React.FC<{ error: Error; resetErrorBoundary: () => void }> = ({ 
  error, 
  resetErrorBoundary 
}) => {
  return (
    <div className="error-screen">
      <h2>Failed to Initialize</h2>
      <p>Error: {error.message}</p>
      <button onClick={resetErrorBoundary}>
        Try Again
      </button>
    </div>
  );
};

const AppCore: React.FC = () => {
  const [isInitialized, setIsInitialized] = useState(false);
  const [initError, setInitError] = useState<string | null>(null);

  useEffect(() => {
    const initializeApp = async () => {
      try {
        if (isTauri) {
          // Tauri-specific initialization
          const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
          const appWindow = getCurrentWebviewWindow();
          
          // Perform Tauri-specific setup
          await appWindow.show();
          await appWindow.setFocus();
        }
        
        setIsInitialized(true);
      } catch (error) {
        console.error('Initialization error:', error);
        setInitError(error instanceof Error ? error.message : 'Unknown error');
      }
    };

    initializeApp();
  }, []);

  if (initError) {
    return (
      <ErrorBoundary FallbackComponent={ErrorFallback}>
        <div className="error-screen">
          <h2>Failed to Initialize</h2>
          <p>Error: {initError}</p>
          <button onClick={() => window.location.reload()}>
            Reload
          </button>
        </div>
      </ErrorBoundary>
    );
  }

  if (!isInitialized) {
    return (
      <div className="loading-screen">
        <div className="spinner" />
        <p>Initializing...</p>
      </div>
    );
  }

  return (
    <ErrorBoundary FallbackComponent={ErrorFallback}>
      <BrowserRouter>
        <Suspense fallback={<div className="loading-screen"><div className="spinner" /><p>Loading...</p></div>}>
          <Routes>
            <Route path="/" element={isTauri ? <Welcome /> : <NonTauriFallback />} />
            <Route path="/dashboard" element={<Dashboard />} />
            <Route path="/settings" element={<Settings />} />
          </Routes>
        </Suspense>
      </BrowserRouter>
    </ErrorBoundary>
  );
};

export default AppCore;
