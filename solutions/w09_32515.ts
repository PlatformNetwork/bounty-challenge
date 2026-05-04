// src/AppCore.tsx
import React, { Suspense, lazy, useEffect, useState } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { ErrorBoundary } from 'react-error-boundary';

// Lazy load components
const Welcome = lazy(() => import('./pages/Welcome'));
const Dashboard = lazy(() => import('./pages/Dashboard'));
const Settings = lazy(() => import('./pages/Settings'));

// Fallback component for loading state
const LoadingFallback = () => (
  <div className="flex items-center justify-center h-screen">
    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
  </div>
);

// Error fallback component
const ErrorFallback = ({ error, resetErrorBoundary }: { error: Error; resetErrorBoundary: () => void }) => (
  <div className="flex flex-col items-center justify-center h-screen bg-gray-100">
    <div className="bg-white p-8 rounded-lg shadow-md max-w-md">
      <h2 className="text-2xl font-bold text-red-600 mb-4">Failed to Initialize</h2>
      <p className="text-gray-700 mb-4">Something went wrong while loading the application.</p>
      <p className="text-sm text-gray-500 mb-4">Error: {error.message}</p>
      <button
        onClick={resetErrorBoundary}
        className="bg-blue-500 hover:bg-blue-600 text-white font-semibold py-2 px-4 rounded"
      >
        Try Again
      </button>
    </div>
  </div>
);

// Safe Tauri API wrapper
const isTauriEnvironment = () => {
  try {
    return typeof window !== 'undefined' && 
           window.__TAURI_INTERNALS__ !== undefined && 
           window.__TAURI_INTERNALS__.metadata !== undefined;
  } catch {
    return false;
  }
};

// Safe webview window accessor
const getSafeWebviewWindow = () => {
  if (isTauriEnvironment()) {
    // Dynamic import to avoid bundling Tauri code in browser builds
    return import('@tauri-apps/api/webviewWindow').then(mod => mod.getCurrentWebviewWindow());
  }
  return Promise.resolve(null);
};

const AppCore: React.FC = () => {
  const [isTauri, setIsTauri] = useState<boolean>(false);
  const [tauriReady, setTauriReady] = useState<boolean>(false);

  useEffect(() => {
    const checkEnvironment = async () => {
      const tauriAvailable = isTauriEnvironment();
      setIsTauri(tauriAvailable);
      
      if (tauriAvailable) {
        try {
          const webviewWindow = await getSafeWebviewWindow();
          if (webviewWindow) {
            // Initialize Tauri-specific features if needed
            setTauriReady(true);
          }
        } catch (error) {
          console.warn('Tauri initialization failed, running in browser mode:', error);
          setIsTauri(false);
          setTauriReady(false);
        }
      } else {
        setTauriReady(false);
      }
    };

    checkEnvironment();
  }, []);

  return (
    <ErrorBoundary FallbackComponent={ErrorFallback}>
      <BrowserRouter>
        <Suspense fallback={<LoadingFallback />}>
          <Routes>
            <Route path="/" element={<Welcome />} />
            <Route path="/dashboard" element={<Dashboard />} />
            <Route path="/settings" element={<Settings />} />
          </Routes>
        </Suspense>
      </BrowserRouter>
    </ErrorBoundary>
  );
};

export default AppCore;
