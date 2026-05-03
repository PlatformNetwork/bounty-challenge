// src/AppCore.tsx
import React, { useEffect, useState } from 'react';
import { isTauri } from './utils/tauriCheck';

// Only import Tauri APIs when running in Tauri
let getCurrentWebviewWindow: any = () => null;
if (isTauri()) {
  // Dynamic import to avoid crash in browser
  import('@tauri-apps/api/webviewWindow').then(mod => {
    getCurrentWebviewWindow = mod.getCurrentWebviewWindow;
  }).catch(() => {
    console.warn('Tauri API import failed, running in browser mode');
  });
}

const AppCore: React.FC = () => {
  const [isInitialized, setIsInitialized] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const initializeApp = async () => {
      try {
        if (isTauri()) {
          // Safe to use Tauri APIs now
          const webviewWindow = getCurrentWebviewWindow();
          if (webviewWindow) {
            // Perform Tauri-specific initialization
            console.log('Running in Tauri environment');
          }
        } else {
          console.log('Running in browser environment');
        }
        
        setIsInitialized(true);
      } catch (err) {
        console.error('Initialization failed:', err);
        setError(err instanceof Error ? err.message : 'Failed to initialize');
      }
    };

    initializeApp();
  }, []);

  if (error) {
    return <div className="error-screen">Failed to Initialize: {error}</div>;
  }

  if (!isInitialized) {
    return <div className="loading-screen">Loading...</div>;
  }

  return <div className="welcome-ui">Welcome to Cortex!</div>;
};

export default AppCore;
