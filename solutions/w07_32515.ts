// src/AppCore.tsx
import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';

// Lazy load Tauri APIs only when needed
const getTauriWindow = async () => {
  try {
    const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
    return getCurrentWebviewWindow();
  } catch {
    return null;
  }
};

// Check if running in Tauri environment
const isTauriEnvironment = (): boolean => {
  try {
    return typeof window !== 'undefined' && 
           window !== null && 
           '__TAURI_INTERNALS__' in window;
  } catch {
    return false;
  }
};

interface AppCoreProps {
  children?: React.ReactNode;
}

const AppCore: React.FC<AppCoreProps> = ({ children }) => {
  const [isInitialized, setIsInitialized] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const navigate = useNavigate();

  useEffect(() => {
    const initializeApp = async () => {
      try {
        // Only access Tauri APIs if in Tauri environment
        if (isTauriEnvironment()) {
          const appWindow = await getTauriWindow();
          if (appWindow) {
            // Use Tauri window APIs safely
            const metadata = window.__TAURI_INTERNALS__.metadata;
            // ... rest of your Tauri-specific initialization
          }
        } else {
          // Browser environment - skip Tauri-specific initialization
          console.log('Running in browser mode, skipping Tauri initialization');
        }

        setIsInitialized(true);
      } catch (err) {
        console.error('Failed to initialize app:', err);
        setError(err instanceof Error ? err.message : 'Unknown error');
      }
    };

    initializeApp();
  }, []);

  if (error) {
    return (
      <div className="error-screen">
        <h2>Failed to Initialize</h2>
        <p>{error}</p>
        <button onClick={() => window.location.reload()}>
          Retry
        </button>
      </div>
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

  return <>{children}</>;
};

export default AppCore;
