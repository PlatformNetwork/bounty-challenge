// src/AppCore.tsx
import React, { useEffect, useState } from 'react';
import { isTauri } from './utils/tauriCheck';

// Only import Tauri APIs when running in Tauri
let getCurrentWebviewWindow: any = () => null;
if (isTauri()) {
  // Dynamic import to avoid crashing in browser
  import('@tauri-apps/api/webviewWindow').then(mod => {
    getCurrentWebviewWindow = mod.getCurrentWebviewWindow;
  }).catch(() => {
    // Silently fail if Tauri APIs are not available
  });
}

interface AppCoreProps {
  children?: React.ReactNode;
}

const AppCore: React.FC<AppCoreProps> = ({ children }) => {
  const [isInitialized, setIsInitialized] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const initializeApp = async () => {
      try {
        // Check if running in Tauri environment
        if (isTauri()) {
          // Access Tauri-specific functionality safely
          const webviewWindow = getCurrentWebviewWindow();
          if (webviewWindow) {
            // Perform Tauri-specific initialization
            console.log('Running in Tauri environment');
          }
        } else {
          // Browser environment - skip Tauri-specific initialization
          console.log('Running in browser environment');
        }

        // Continue with normal app initialization
        setIsInitialized(true);
      } catch (err) {
        console.error('Initialization error:', err);
        setError(err instanceof Error ? err.message : 'Failed to initialize');
      }
    };

    initializeApp();
  }, []);

  if (error) {
    return (
      <div className="error-screen">
        <h2>Failed to Initialize</h2>
        <p>{error}</p>
        <button onClick={() => window.location.reload()}>Retry</button>
      </div>
    );
  }

  if (!isInitialized) {
    return <div className="loading-screen">Loading...</div>;
  }

  return <>{children}</>;
};

export default AppCore;
