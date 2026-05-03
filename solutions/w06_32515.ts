// src/AppCore.tsx
import React, { Suspense, lazy, useEffect, useState } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { ErrorBoundary } from 'react-error-boundary';

// Dynamic import with fallback for non-Tauri environments
const TauriWindow = React.lazy(() => 
  import('@tauri-apps/api/window').catch(() => ({
    default: {
      getCurrentWebviewWindow: () => ({
        label: 'main',
        listen: () => () => {},
        emit: () => {},
        onResized: () => () => {},
        onMoved: () => () => {},
        onCloseRequested: () => () => {},
        onFocusChanged: () => () => {},
        onBlurred: () => () => {},
        onFocused: () => () => {},
        onScaleChanged: () => () => {},
        onThemeChanged: () => () => {},
        center: () => Promise.resolve(),
        close: () => Promise.resolve(),
        hide: () => Promise.resolve(),
        show: () => Promise.resolve(),
        maximize: () => Promise.resolve(),
        unmaximize: () => Promise.resolve(),
        minimize: () => Promise.resolve(),
        unminimize: () => Promise.resolve(),
        setFocus: () => Promise.resolve(),
        setSize: () => Promise.resolve(),
        setMinSize: () => Promise.resolve(),
        setMaxSize: () => Promise.resolve(),
        setResizable: () => Promise.resolve(),
        setTitle: () => Promise.resolve(),
        setAlwaysOnTop: () => Promise.resolve(),
        setDecorations: () => Promise.resolve(),
        setFullscreen: () => Promise.resolve(),
        setCursorGrab: () => Promise.resolve(),
        setCursorIcon: () => Promise.resolve(),
        setCursorPosition: () => Promise.resolve(),
        setIgnoreCursorEvents: () => Promise.resolve(),
        startDragging: () => Promise.resolve(),
        innerPosition: () => Promise.resolve({ x: 0, y: 0 }),
        outerPosition: () => Promise.resolve({ x: 0, y: 0 }),
        innerSize: () => Promise.resolve({ width: 0, height: 0 }),
        outerSize: () => Promise.resolve({ width: 0, height: 0 }),
        isFullscreen: () => Promise.resolve(false),
        isMaximized: () => Promise.resolve(false),
        isMinimized: () => Promise.resolve(false),
        isDecorated: () => Promise.resolve(false),
        isResizable: () => Promise.resolve(false),
        isVisible: () => Promise.resolve(false),
        scaleFactor: () => Promise.resolve(1),
        theme: () => Promise.resolve('light'),
      }),
    },
  }))
);

// Check if running in Tauri environment
const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

// Safe wrapper for Tauri window operations
const useSafeWindow = () => {
  const [windowAPI, setWindowAPI] = useState(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const loadWindowAPI = async () => {
      if (isTauri) {
        try {
          const { getCurrentWebviewWindow } = await import('@tauri-apps/api/window');
          setWindowAPI(getCurrentWebviewWindow());
        } catch (error) {
          console.warn('Failed to load Tauri window API:', error);
          setWindowAPI(null);
        }
      }
      setIsLoading(false);
    };

    loadWindowAPI();
  }, []);

  return { windowAPI, isLoading };
};

// Error fallback component
const ErrorFallback = ({ error, resetErrorBoundary }) => (
  <div style={{ padding: '20px', textAlign: 'center' }}>
    <h2>Something went wrong</h2>
    <pre style={{ color: 'red' }}>{error.message}</pre>
    <button onClick={resetErrorBoundary}>Try again</button>
  </div>
);

// Lazy load main app components
const WelcomeScreen = lazy(() => import('./WelcomeScreen'));
const Dashboard = lazy(() => import('./Dashboard'));

// Loading component
const LoadingScreen = () => (
  <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh' }}>
    <div>Loading...</div>
  </div>
);

// Main AppCore component
const AppCore: React.FC = () => {
  const { windowAPI, isLoading } = useSafeWindow();

  if (isLoading) {
    return <LoadingScreen />;
  }

  return (
    <ErrorBoundary FallbackComponent={ErrorFallback}>
      <BrowserRouter>
        <Suspense fallback={<LoadingScreen />}>
          <Routes>
            <Route path="/" element={<WelcomeScreen windowAPI={windowAPI} />} />
            <Route path="/dashboard" element={<Dashboard windowAPI={windowAPI} />} />
          </Routes>
        </Suspense>
      </BrowserRouter>
    </ErrorBoundary>
  );
};

export default AppCore;
