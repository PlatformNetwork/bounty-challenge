// src/AppCore.tsx
import React, { lazy, Suspense, useEffect, useState } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';

// Check if running in Tauri environment
const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;

// Lazy load components
const Welcome = lazy(() => import('./pages/Welcome'));
const Dashboard = lazy(() => import('./pages/Dashboard'));
const Settings = lazy(() => import('./pages/Settings'));

// Safe wrapper for Tauri-specific functionality
const useTauriWindow = () => {
  const [windowInfo, setWindowInfo] = useState(null);

  useEffect(() => {
    if (isTauri) {
      // Dynamically import Tauri APIs only when in Tauri environment
      import('@tauri-apps/api/window').then(({ getCurrentWebviewWindow }) => {
        const appWindow = getCurrentWebviewWindow();
        setWindowInfo({
          label: appWindow.label,
          // Add other window properties as needed
        });
      }).catch((err) => {
        console.warn('Failed to load Tauri window API:', err);
      });
    }
  }, []);

  return windowInfo;
};

// Error boundary component
class ErrorBoundary extends React.Component {
  constructor(props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error) {
    return { hasError: true, error };
  }

  componentDidCatch(error, errorInfo) {
    console.error('AppCore Error:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="error-screen">
          <h2>Failed to Initialize</h2>
          <p>An error occurred while loading the application.</p>
          <button onClick={() => window.location.reload()}>
            Reload Application
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}

// Loading component
const LoadingScreen = () => (
  <div className="loading-screen">
    <div className="spinner"></div>
    <p>Loading Cortex...</p>
  </div>
);

// Main AppCore component
const AppCore: React.FC = () => {
  const tauriWindow = useTauriWindow();
  const [isInitialized, setIsInitialized] = useState(false);

  useEffect(() => {
    // Initialize app without Tauri dependency
    const initApp = async () => {
      try {
        // Perform any necessary initialization
        if (!isTauri) {
          console.log('Running in browser mode');
        }
        setIsInitialized(true);
      } catch (error) {
        console.error('Initialization failed:', error);
        setIsInitialized(false);
      }
    };

    initApp();
  }, []);

  if (!isInitialized) {
    return <LoadingScreen />;
  }

  return (
    <ErrorBoundary>
      <BrowserRouter>
        <Suspense fallback={<LoadingScreen />}>
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
