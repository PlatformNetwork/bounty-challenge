// src/AppCore.tsx
import React, { lazy, Suspense, useEffect, useState } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';

// Check if running in Tauri environment
const isTauri = typeof window !== 'undefined' && 
  window.__TAURI_INTERNALS__ !== undefined;

// Lazy load components
const Welcome = lazy(() => import('./pages/Welcome'));
const Dashboard = lazy(() => import('./pages/Dashboard'));
const Settings = lazy(() => import('./pages/Settings'));

// Error boundary component
class ErrorBoundary extends React.Component<
  { children: React.ReactNode },
  { hasError: boolean; error: Error | null }
> {
  constructor(props: { children: React.ReactNode }) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="error-screen">
          <h2>Failed to Initialize</h2>
          <p>{this.state.error?.message}</p>
          <button onClick={() => window.location.reload()}>
            Retry
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}

// Safe Tauri API wrapper
const tauriAPI = {
  getWindow: () => {
    if (!isTauri) {
      return {
        label: 'main',
        listen: () => Promise.resolve(() => {}),
        emit: () => Promise.resolve(),
        onResized: () => Promise.resolve(() => {}),
        onMoved: () => Promise.resolve(() => {}),
        onCloseRequested: () => Promise.resolve(() => {}),
        onFocusChanged: () => Promise.resolve(() => {}),
        onScaleChanged: () => Promise.resolve(() => {}),
        onThemeChanged: () => Promise.resolve(() => {}),
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
        setSkipTaskbar: () => Promise.resolve(),
        setCursorGrab: () => Promise.resolve(),
        setCursorVisible: () => Promise.resolve(),
        setCursorIcon: () => Promise.resolve(),
        setCursorPosition: () => Promise.resolve(),
        setIgnoreCursorEvents: () => Promise.resolve(),
        startDragging: () => Promise.resolve(),
        innerPosition: () => Promise.resolve({ x: 0, y: 0 }),
        outerPosition: () => Promise.resolve({ x: 0, y: 0 }),
        innerSize: () => Promise.resolve({ width: 0, height: 0 }),
        outerSize: () => Promise.resolve({ width: 0, height: 0 }),
        isFullscreen: () => Promise.resolve(false),
        isDecorated: () => Promise.resolve(false),
        isResizable: () => Promise.resolve(false),
        isMaximized: () => Promise.resolve(false),
        isMinimized: () => Promise.resolve(false),
        isFocused: () => Promise.resolve(false),
        isVisible: () => Promise.resolve(false),
        scaleFactor: () => Promise.resolve(1),
        theme: () => Promise.resolve('light'),
      };
    }
    
    // Dynamic import to avoid crash in browser
    try {
      const { getCurrentWebviewWindow } = window.__TAURI_INTERNALS__;
      return getCurrentWebviewWindow();
    } catch {
      return null;
    }
  },
  
  invoke: async (cmd: string, args?: Record<string, unknown>) => {
    if (!isTauri) {
      console.warn(`Tauri invoke called in browser: ${cmd}`);
      return null;
    }
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return invoke(cmd, args);
    } catch {
      console.warn(`Failed to invoke Tauri command: ${cmd}`);
      return null;
    }
  },
  
  listen: async (event: string, handler: (...args: unknown[]) => void) => {
    if (!isTauri) {
      console.warn(`Tauri listen called in browser: ${event}`);
      return () => {};
    }
    
    try {
      const { listen } = await import('@tauri-apps/api/event');
      return listen(event, handler);
    } catch {
      console.warn(`Failed to listen to Tauri event: ${event}`);
      return () => {};
    }
  },
  
  emit: async (event: string, payload?: unknown) => {
    if (!isTauri) {
      console.warn(`Tauri emit called in browser: ${event}`);
      return;
    }
    
    try {
      const { emit } = await import('@tauri-apps/api/event');
      return emit(event, payload);
    } catch {
      console.warn(`Failed to emit Tauri event: ${event}`);
    }
  },
  
  getMetadata: () => {
    if (!isTauri) {
      return {
        appName: 'Cortex',
        appVersion: '1.0.0',
        tauriVersion: '2.0.0',
        platform: 'web',
        arch: 'x86_64',
      };
    }
    
    try {
      return window.__TAURI_INTERNALS__.metadata;
    } catch {
      return {
        appName: 'Cortex',
        appVersion: '1.0.0',
        tauriVersion: '2.0.0',
        platform: 'web',
        arch: 'x86_64',
      };
    }
  },
};

// Export the safe API
export { tauriAPI, isTauri };

// AppCore component
const AppCore: React.FC = () => {
  const [initialized, setInitialized] = useState(false);
  const [initError, setInitError] = useState<string | null>(null);

  useEffect(() => {
    const init = async () => {
      try {
        // Check if we can safely access Tauri APIs
        if (isTauri) {
          const metadata = tauriAPI.getMetadata();
          console.log('Running in Tauri:', metadata);
        } else {
          console.log('Running in browser');
        }
        
        setInitialized(true);
      } catch (error) {
        console.error('Initialization error:', error);
        setInitError(error instanceof Error ? error.message : 'Unknown error');
      }
    };

    init();
  }, []);

  if (initError) {
    return (
      <div className="error-screen">
        <h2>Failed to Initialize</h2>
        <p>{initError}</p>
        <button onClick={() => window.location.reload()}>
          Retry
        </button>
      </div>
    );
  }

  if (!initialized) {
    return (
      <div className="loading-screen">
        <div className="spinner" />
        <p>Initializing...</p>
      </div>
    );
  }

  return (
    <ErrorBoundary>
      <BrowserRouter>
        <Suspense fallback={
          <div className="loading-screen">
            <div className="spinner" />
            <p>Loading...</p>
          </div>
        }>
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
