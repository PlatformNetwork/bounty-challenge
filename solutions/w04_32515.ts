// src/AppCore.tsx
import React, { Suspense, lazy, useEffect, useState } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { ErrorBoundary } from 'react-error-boundary';

// Lazy load components
const Welcome = lazy(() => import('./pages/Welcome'));
const Dashboard = lazy(() => import('./pages/Dashboard'));
const Settings = lazy(() => import('./pages/Settings'));

// Check if running in Tauri environment
const isTauri = () => {
  try {
    return window.__TAURI__ !== undefined;
  } catch {
    return false;
  }
};

// Safe Tauri API wrapper
const safeTauriAPI = {
  getCurrentWebviewWindow: () => {
    if (isTauri()) {
      // Dynamic import to avoid static import issues
      return import('@tauri-apps/api/webviewWindow').then(mod => mod.getCurrentWebviewWindow());
    }
    return null;
  },
  getMetadata: () => {
    if (isTauri() && window.__TAURI_INTERNALS__?.metadata) {
      return window.__TAURI_INTERNALS__.metadata;
    }
    return null;
  }
};

// Error Fallback Component
const ErrorFallback = ({ error, resetErrorBoundary }) => {
  return (
    <div className="error-screen">
      <h2>Failed to Initialize</h2>
      <p>Something went wrong while loading the application.</p>
      <pre>{error?.message}</pre>
      <button onClick={resetErrorBoundary}>Try Again</button>
    </div>
  );
};

// Loading Component
const LoadingScreen = () => (
  <div className="loading-screen">
    <div className="spinner"></div>
    <p>Loading Cortex...</p>
  </div>
);

// Main App Component
const AppCore: React.FC = () => {
  const [isInitialized, setIsInitialized] = useState(false);
  const [initError, setInitError] = useState<Error | null>(null);

  useEffect(() => {
    const initializeApp = async () => {
      try {
        // Check if we're in a browser environment
        if (!isTauri()) {
          console.log('Running in browser mode - Tauri APIs disabled');
          setIsInitialized(true);
          return;
        }

        // Initialize Tauri-specific features safely
        const webviewWindow = await safeTauriAPI.getCurrentWebviewWindow();
        if (webviewWindow) {
          console.log('Tauri webview initialized successfully');
        }

        setIsInitialized(true);
      } catch (error) {
        console.error('Initialization error:', error);
        setInitError(error as Error);
      }
    };

    initializeApp();
  }, []);

  if (initError) {
    return (
      <ErrorBoundary FallbackComponent={ErrorFallback}>
        <ErrorFallback error={initError} resetErrorBoundary={() => setInitError(null)} />
      </ErrorBoundary>
    );
  }

  if (!isInitialized) {
    return <LoadingScreen />;
  }

  return (
    <ErrorBoundary FallbackComponent={ErrorFallback}>
      <Suspense fallback={<LoadingScreen />}>
        <BrowserRouter>
          <Routes>
            <Route path="/" element={<Welcome />} />
            <Route path="/dashboard" element={<Dashboard />} />
            <Route path="/settings" element={<Settings />} />
          </Routes>
        </BrowserRouter>
      </Suspense>
    </ErrorBoundary>
  );
};

export default AppCore;
