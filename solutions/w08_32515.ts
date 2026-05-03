// src/AppCore.tsx
import React, { Suspense, lazy, useEffect, useState } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { ErrorBoundary } from 'react-error-boundary';

// Lazy load components
const Welcome = lazy(() => import('./pages/Welcome'));
const Dashboard = lazy(() => import('./pages/Dashboard'));
const Settings = lazy(() => import('./pages/Settings'));

// Safe Tauri API wrapper
const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;

// Fallback component for loading state
const LoadingFallback = () => (
  <div className="flex items-center justify-center min-h-screen">
    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
  </div>
);

// Error fallback component
const ErrorFallback = ({ error, resetErrorBoundary }: { error: Error; resetErrorBoundary: () => void }) => (
  <div className="flex flex-col items-center justify-center min-h-screen p-8">
    <h1 className="text-2xl font-bold text-red-600 mb-4">Failed to Initialize</h1>
    <p className="text-gray-600 mb-4">{error.message}</p>
    <button
      onClick={resetErrorBoundary}
      className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
    >
      Try Again
    </button>
  </div>
);

// Safe window metadata access
const getWindowMetadata = () => {
  if (!isTauri) {
    return {
      label: 'main',
      url: window.location.href,
      title: document.title,
      width: window.innerWidth,
      height: window.innerHeight,
    };
  }

  try {
    // @ts-ignore - Tauri API
    const metadata = window.__TAURI_INTERNALS__.metadata;
    if (metadata) {
      return metadata;
    }
  } catch (e) {
    console.warn('Failed to access Tauri metadata:', e);
  }

  return null;
};

// Safe webview window access
const getCurrentWebviewWindow = () => {
  if (!isTauri) {
    return {
      label: 'main',
      minimize: async () => {},
      maximize: async () => {},
      unmaximize: async () => {},
      close: async () => {},
      setSize: async () => {},
      setTitle: async () => {},
      onResized: async () => {},
      onMoved: async () => {},
      onCloseRequested: async () => {},
      onFocusChanged: async () => {},
      onScaleChanged: async () => {},
      onThemeChanged: async () => {},
      onFileDropEvent: async () => {},
      onDragDropEvent: async () => {},
      listen: async () => () => {},
      once: async () => {},
      emit: async () => {},
    };
  }

  try {
    // @ts-ignore - Tauri API
    const { getCurrentWebviewWindow } = window.__TAURI__.webviewWindow;
    return getCurrentWebviewWindow();
  } catch (e) {
    console.warn('Failed to get current webview window:', e);
    return null;
  }
};

// Safe invoke wrapper
const invoke = async (cmd: string, args?: Record<string, unknown>) => {
  if (!isTauri) {
    console.warn(`Tauri command "${cmd}" called outside Tauri environment`);
    return null;
  }

  try {
    // @ts-ignore - Tauri API
    return await window.__TAURI__.invoke(cmd, args);
  } catch (e) {
    console.warn(`Failed to invoke Tauri command "${cmd}":`, e);
    return null;
  }
};

// Safe event listener
const listen = async (event: string, handler: (...args: unknown[]) => void) => {
  if (!isTauri) {
    console.warn(`Tauri event listener "${event}" registered outside Tauri environment`);
    return () => {};
  }

  try {
    // @ts-ignore - Tauri API
    return await window.__TAURI__.event.listen(event, handler);
  } catch (e) {
    console.warn(`Failed to listen to Tauri event "${event}":`, e);
    return () => {};
  }
};

// App Core Component
const AppCore: React.FC = () => {
  const [isInitialized, setIsInitialized] = useState(false);
  const [initError, setInitError] = useState<Error | null>(null);

  useEffect(() => {
    const initializeApp = async () => {
      try {
        // Check if we're in a browser environment
        if (!isTauri) {
          console.log('Running in browser mode - skipping Tauri initialization');
          setIsInitialized(true);
          return;
        }

        // Try to access Tauri metadata safely
        const metadata = getWindowMetadata();
        if (!metadata) {
          console.warn('Tauri metadata not available - running in browser mode');
          setIsInitialized(true);
          return;
        }

        // Initialize Tauri-specific features
        const webviewWindow = getCurrentWebviewWindow();
        if (webviewWindow) {
          // Set up window event listeners
          await listen('tauri://close-requested', () => {
            console.log('Close requested');
          });

          await listen('tauri://resize', () => {
            console.log('Window resized');
          });
        }

        setIsInitialized(true);
      } catch (error) {
        console.error('Failed to initialize app:', error);
        setInitError(error instanceof Error ? error : new Error('Unknown initialization error'));
      }
    };

    initializeApp();
  }, []);

  if (initError) {
    return (
      <ErrorBoundary FallbackComponent={ErrorFallback} onReset={() => setInitError(null)}>
        <ErrorFallback error={initError} resetErrorBoundary={() => setInitError(null)} />
      </ErrorBoundary>
    );
  }

  if (!isInitialized) {
    return <LoadingFallback />;
  }

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

export { AppCore, isTauri, getWindowMetadata, getCurrentWebviewWindow, invoke, listen };
export default AppCore;
